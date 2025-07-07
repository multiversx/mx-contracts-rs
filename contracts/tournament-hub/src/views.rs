use crate::models::{GameConfig, SpectatorBet, Tournament};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ViewsModule: crate::storage::StorageModule {
    #[view(getGameConfig)]
    fn get_game_config(&self, game_id: &ManagedBuffer) -> Option<GameConfig<Self::Api>> {
        if self.registered_games().contains_key(game_id) {
            Some(self.registered_games().get(game_id).unwrap())
        } else {
            None
        }
    }

    #[view(getTournament)]
    fn get_tournament(&self, tournament_id: &ManagedBuffer) -> Option<Tournament<Self::Api>> {
        if self.active_tournaments().contains_key(tournament_id) {
            Some(self.active_tournaments().get(tournament_id).unwrap())
        } else {
            None
        }
    }

    #[view(getSpectatorBets)]
    fn get_spectator_bets(
        &self,
        tournament_id: &ManagedBuffer,
        player: &ManagedAddress,
    ) -> ManagedVec<SpectatorBet<Self::Api>> {
        self.spectator_bets(tournament_id, player).get()
    }

    #[view(getSpectatorPoolTotal)]
    fn get_spectator_pool_total(&self, tournament_id: &ManagedBuffer) -> BigUint {
        self.spectator_pool_total(tournament_id).get()
    }
}
