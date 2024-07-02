use multiversx_sc::imports::*;

use crate::types::GameSettings;

#[multiversx_sc::module]
pub trait StorageModule {
    //GENERAL SETTINGS
    #[view(getTokenId)]
    #[storage_mapper("tokenId")]
    fn token_id(&self) -> SingleValueMapper<EgldOrEsdtTokenIdentifier>;

    #[view(getGameStartFee)]
    #[storage_mapper("gameStartFee")]
    fn game_start_fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getEnabled)]
    #[storage_mapper("enabled")]
    fn enabled(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("admins")]
    fn admins(&self) -> WhitelistMapper<ManagedAddress>;

    #[view(isUserAdmin)]
    fn is_user_admin(&self, user: ManagedAddress) -> bool {
        self.admins().contains(&user)
    }

    //GAME
    #[view(getLastGameId)]
    #[storage_mapper("lastGameId")]
    fn last_game_id(&self) -> SingleValueMapper<u64>;

    #[view(getGameSettings)]
    #[storage_mapper("gameSettings")]
    fn game_settings(&self, game_id: u64) -> SingleValueMapper<GameSettings<Self::Api>>;

    #[view(getGameIdBySettings)]
    #[storage_mapper("gameIdBySettings")]
    fn game_id(&self, game_settings: &GameSettings<Self::Api>) -> SingleValueMapper<u64>;

    #[view(getPlayers)]
    #[storage_mapper("players")]
    fn players(&self, game_id: u64) -> UnorderedSetMapper<ManagedAddress>;

    //USERS
    #[view(getGamesPerUser)]
    #[storage_mapper("gamesPerUser")]
    fn games_per_user(&self, user: &ManagedAddress) -> UnorderedSetMapper<u64>;
}
