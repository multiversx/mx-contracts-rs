use crate::models::{GameConfig, SpectatorBet, Tournament};
use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait StorageModule {
    #[storage_mapper("owner")]
    fn owner(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("registered_games")]
    fn registered_games(&self) -> MapMapper<ManagedBuffer, GameConfig<Self::Api>>;

    #[storage_mapper("active_tournaments")]
    fn active_tournaments(&self) -> MapMapper<ManagedBuffer, Tournament<Self::Api>>;

    #[storage_mapper("spectator_bets")]
    fn spectator_bets(
        &self,
        tournament_id: &ManagedBuffer,
        player: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<SpectatorBet<Self::Api>>>;

    #[storage_mapper("spectator_pool_total")]
    fn spectator_pool_total(&self, tournament_id: &ManagedBuffer) -> SingleValueMapper<BigUint>;

    #[storage_mapper("spectator_claims")]
    fn spectator_claims(&self) -> MapMapper<ManagedBuffer, bool>;
}
