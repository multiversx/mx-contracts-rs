#![no_std]

use multiversx_sc::imports::*;

pub mod game_proxy;
pub mod owner;
pub mod private;
pub mod storage;
pub mod types;

#[multiversx_sc::contract]
pub trait MvxGameSc: storage::StorageModule + owner::OwnerModule + private::PrivateModule {
    #[allow_multiple_var_args]
    #[init]
    fn init(
        &self,
        enabled_opt: OptionalValue<bool>,
        game_start_fee_opt: OptionalValue<BigUint>,
        token_id_opt: OptionalValue<EgldOrEsdtTokenIdentifier>,
    ) {
        match enabled_opt {
            OptionalValue::Some(_) => self.enabled().set(true),
            OptionalValue::None => {}
        }

        match game_start_fee_opt {
            OptionalValue::Some(val) => self.game_start_fee().set(val),
            OptionalValue::None => {
                require!(!self.game_start_fee().is_empty(), "game start fee not set")
            }
        }

        match token_id_opt {
            OptionalValue::Some(val) => self.token_id().set(val),
            OptionalValue::None => require!(!self.token_id().is_empty(), "fee token id not set"),
        }
    }

    #[payable("*")]
    #[endpoint(createGame)]
    fn create_game(
        &self,
        waiting_time: u64,
        number_of_players_min: u64,
        number_of_players_max: u64,
        wager: BigUint,
    ) -> u64 {
        self.require_enabled();

        let (token_id, amount) = self.call_value().single_fungible_esdt();
        self.validate_create_game_payment(&token_id, &amount, &wager, waiting_time);

        let (min, max) = self.get_min_max(number_of_players_min, number_of_players_max);

        let caller = self.blockchain().get_caller();

        self.create_new_game(caller, waiting_time, min, max, wager)
    }

    #[payable("*")]
    #[endpoint(joinGame)]
    fn join_game(&self, game_id: u64) {
        self.require_enabled();

        let (token_id, amount) = self.call_value().single_fungible_esdt();
        let now = self.blockchain().get_block_timestamp();
        let caller = self.blockchain().get_caller();

        let game_settings = self.validate_join_game(&caller, now, &token_id, &amount, game_id);

        self.add_player(caller, game_id);

        self.refresh_game_status(game_id, game_settings);
    }

    //manually claim back wager if the game is invalid
    #[endpoint(claimBackWager)]
    fn claim_back_wager(&self, game_id: u64) {
        self.require_enabled();

        let caller = self.blockchain().get_caller();
        let wager = self.validate_claim_wager(&caller, game_id);

        let token_id = self.token_id().get();
        self.tx()
            .to(&caller)
            .egld_or_single_esdt(&token_id, 0, &wager)
            .transfer();
        self.remove_player(caller, game_id);
    }
}
