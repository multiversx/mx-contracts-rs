use crate::types::{GameSettings, Status};

use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait PrivateModule: crate::storage::StorageModule {
    //game
    fn create_new_game(
        &self,
        caller: ManagedAddress,
        waiting_time: u64,
        min: u64,
        max: u64,
        wager: BigUint,
    ) -> u64 {
        let new_id = self.get_new_game_id();
        self.last_game_id().set(new_id);
        let now = self.blockchain().get_block_timestamp();

        let time_limit = now + waiting_time;
        let game_settings = GameSettings {
            time_limit,
            number_of_players_min: min,
            number_of_players_max: max,
            wager,
            creator: caller,
            status: Status::Invalid,
        };

        self.game_id(&game_settings).set(new_id);
        self.game_settings(new_id).set(game_settings);

        new_id
    }

    fn add_player(&self, caller: ManagedAddress, game_id: u64) {
        self.games_per_user(&caller).insert(game_id);
        self.players(game_id).insert(caller);
    }

    fn remove_player(&self, caller: ManagedAddress, game_id: u64) {
        self.games_per_user(&caller).swap_remove(&game_id);
        self.players(game_id).swap_remove(&caller);
    }

    fn refresh_game_status(&self, game_id: u64, game_settings: GameSettings<Self::Api>) {
        let len = self.players(game_id).len() as u64;
        if game_settings.number_of_players_min <= len {
            self.game_settings(game_id)
                .update(|val| val.status = Status::Valid);
        }
    }

    fn send_back_wager(&self, game_id: u64, wager: &BigUint, token_id: &EgldOrEsdtTokenIdentifier) {
        for player in self.players(game_id).iter() {
            self.tx()
                .to(&player)
                .egld_or_single_esdt(token_id, 0, wager)
                .transfer();
        }
    }

    //requires
    fn validate_create_game_payment(
        &self,
        token_id: &TokenIdentifier,
        amount: &BigUint,
        wager: &BigUint,
        waiting_time: u64,
    ) {
        require!(wager > &BigUint::zero(), "wager can't be 0");
        require!(waiting_time > 0u64, "waiting time can't be 0");

        let approved_token_id = self.token_id().get();
        let start_fee = self.game_start_fee().get();

        require!(token_id == &approved_token_id, "wrong token id");
        require!(amount == &start_fee, "start game payment amount not right");
    }

    fn validate_join_game(
        &self,
        caller: &ManagedAddress,
        now: u64,
        token_id: &TokenIdentifier,
        amount: &BigUint,
        game_id: u64,
    ) -> GameSettings<Self::Api> {
        require!(
            !self.game_settings(game_id).is_empty(),
            "no settings for game id"
        );
        let game_settings = self.game_settings(game_id).get();
        let accepted_token_id = self.token_id().get();

        require!(
            !self.games_per_user(caller).contains(&game_id),
            "user already joined this game"
        );

        require!(now <= game_settings.time_limit, "waiting time has passed");

        let len = self.players(game_id).len() as u64;
        require!(
            len < game_settings.number_of_players_max,
            "max number of players reached"
        );

        require!(token_id == &accepted_token_id, "wrong token sent");
        require!(amount == &game_settings.wager, "wrong amount paid");

        game_settings
    }

    fn validate_claim_wager(&self, caller: &ManagedAddress, game_id: u64) -> BigUint {
        require!(
            !self.game_settings(game_id).is_empty(),
            "no settings for game id"
        );

        require!(
            self.games_per_user(caller).contains(&game_id),
            "caller has not joined the game"
        );

        let game_settings = self.game_settings(game_id).get();
        let now = self.blockchain().get_block_timestamp();

        require!(
            now > game_settings.time_limit,
            "waiting time is not over yet"
        );

        require!(
            game_settings.status == Status::Invalid,
            "can manually claim back wager only if the game is invalid"
        );

        game_settings.wager
    }

    fn validate_send_reward(&self, game_id: u64) -> GameSettings<Self::Api> {
        require!(
            !self.game_settings(game_id).is_empty(),
            "no settings for game id"
        );

        let game_settings = self.game_settings(game_id).get();
        let now = self.blockchain().get_block_timestamp();

        require!(
            now > game_settings.time_limit,
            "waiting time is not over yet"
        );

        game_settings
    }

    fn require_enabled(&self) {
        require!(!self.enabled().is_empty(), "maintenance")
    }

    //helper
    fn get_new_game_id(&self) -> u64 {
        if self.last_game_id().is_empty() {
            return 1u64;
        }
        let last_id = self.last_game_id().get();
        last_id + 1u64
    }

    fn get_min_max(&self, a: u64, b: u64) -> (u64, u64) {
        require!(a != 0u64 && b != 0u64, "number of players cannot be 0");

        if a > b {
            return (b, a);
        }

        (a, b)
    }
}
