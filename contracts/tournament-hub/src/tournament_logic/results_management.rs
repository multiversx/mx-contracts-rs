use crate::models::TournamentStatus;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ResultsManagementModule:
    crate::storage::StorageModule + crate::helpers::HelperModule
{
    #[endpoint(submitResults)]
    fn submit_results(
        &self,
        tournament_id: ManagedBuffer,
        winner_podium: ManagedVec<ManagedAddress>,
        signed_result: ManagedBuffer,
    ) {
        require!(
            self.active_tournaments().contains_key(&tournament_id),
            "Tournament does not exist"
        );

        let mut tournament = self.active_tournaments().get(&tournament_id).unwrap();
        let game_config = self.registered_games().get(&tournament.game_id).unwrap();

        require!(
            tournament.status == TournamentStatus::Playing,
            "Tournament is not in playing phase"
        );

        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time >= tournament.play_deadline,
            "Play deadline has not passed yet"
        );

        // Verify signature (simplified - in production, implement proper signature verification)
        // This would involve verifying the signed_result against the game's signing_server_address
        self.verify_result_signature(&tournament_id, &winner_podium, &signed_result, &game_config);

        // Validate winner podium
        require!(
            winner_podium.len() == game_config.podium_size as usize,
            "Winner podium size mismatch"
        );

        // Verify all winners are participants
        for winner in winner_podium.iter() {
            let mut found = false;
            for participant in tournament.participants.iter() {
                if participant == winner {
                    found = true;
                    break;
                }
            }
            require!(found, "Winner not found in participants");
        }

        tournament.status = TournamentStatus::ProcessingResults;
        tournament.final_podium = winner_podium.clone();

        // Calculate and distribute prizes
        self.distribute_player_prizes(&tournament, &game_config);

        tournament.status = TournamentStatus::Completed;
        self.active_tournaments().insert(tournament_id, tournament);
    }
}
