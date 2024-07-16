use crate::types::Status;

use multiversx_sc::imports::*;

const DENOM: u64 = 10_000u64;

#[multiversx_sc::module]
pub trait OwnerModule: crate::private::PrivateModule + crate::storage::StorageModule {
    //u64 is percentage * 100
    //function called by the owner/admins when the winners have been decided
    #[endpoint(sendReward)]
    fn send_reward(
        &self,
        game_id: u64,
        winners: OptionalValue<MultiValueEncoded<(ManagedAddress, u64)>>,
    ) {
        self.require_enabled();

        let caller = self.blockchain().get_caller();
        self.admins().require_whitelisted(&caller);

        let game_settings = self.validate_send_reward(game_id);
        let token_id = self.token_id().get();

        match game_settings.status {
            Status::Invalid => {
                self.send_back_wager(game_id, &game_settings.wager, &token_id);

                let game_creation_fee = self.game_start_fee().get();
                self.tx()
                    .to(game_settings.creator)
                    .egld_or_single_esdt(&token_id, 0, &game_creation_fee)
                    .transfer();

                self.game_settings(game_id).clear();
            }
            Status::Valid => {
                match winners {
                    OptionalValue::Some(val) => {
                        let len = self.players(game_id).len();
                        let total_wager = &BigUint::from(len) * &game_settings.wager;

                        for (winner, percentage) in val.into_iter() {
                            let reward_per_winner =
                                &BigUint::from(percentage) * &total_wager / &BigUint::from(DENOM);
                            self.tx()
                                .to(winner)
                                .egld_or_single_esdt(&token_id, 0, &reward_per_winner)
                                .transfer();
                        }
                    }
                    //tie/draw
                    OptionalValue::None => {
                        self.send_back_wager(game_id, &game_settings.wager, &token_id);
                    }
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(enableSC)]
    fn enable_sc(&self) {
        self.enabled().set(true)
    }

    #[only_owner]
    #[endpoint(disableSC)]
    fn disable_sc(&self) {
        self.enabled().clear()
    }

    #[only_owner]
    #[endpoint(setTokenId)]
    fn set_token_id(&self, token_id: EgldOrEsdtTokenIdentifier) {
        self.token_id().set(token_id)
    }

    #[only_owner]
    #[endpoint(setGameStartFee)]
    fn set_game_start_fee(&self, amount: BigUint) {
        self.game_start_fee().set(amount)
    }

    #[only_owner]
    #[endpoint(setAdmin)]
    fn set_admin(&self, user: ManagedAddress) {
        self.admins().add(&user)
    }

    #[only_owner]
    #[endpoint(removeAdmin)]
    fn remove_admin(&self, user: ManagedAddress) {
        self.admins().remove(&user)
    }
}
