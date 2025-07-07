use crate::models::*;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait SpectatorBettingModule:
    crate::storage::StorageModule + crate::helpers::HelperModule
{
    #[endpoint(placeSpectatorBet)]
    #[payable("EGLD")]
    fn place_spectator_bet(&self, tournament_id: ManagedBuffer, betting_on_player: ManagedAddress) {
        let payment = self.call_value().egld().clone_value();
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(payment > 0, "Bet amount must be greater than 0");
        require!(
            self.active_tournaments().contains_key(&tournament_id),
            "Tournament does not exist"
        );

        let tournament = match self.active_tournaments().get(&tournament_id) {
            Some(t) => t,
            None => sc_panic!("Tournament does not exist"),
        };

        require!(
            current_time <= tournament.play_deadline,
            "Betting period has ended"
        );

        // Verify betting_on_player is a participant
        let found = tournament
            .participants
            .iter()
            .any(|participant| participant.clone_value() == betting_on_player);
        require!(found, "Player not found in tournament");

        // Create bet
        let bet = SpectatorBet {
            bettor_address: caller.clone(),
            amount: payment.clone(),
        };

        // Store bet
        let mut bets = self
            .spectator_bets(&tournament_id, &betting_on_player)
            .get();
        bets.push(bet);
        self.spectator_bets(&tournament_id, &betting_on_player)
            .set(bets);

        // Update total spectator pool
        let current_pool = self.spectator_pool_total(&tournament_id).get();
        self.spectator_pool_total(&tournament_id)
            .set(current_pool + payment);
    }

    #[endpoint(claimSpectatorWinnings)]
    fn claim_spectator_winnings(&self, tournament_id: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(
            self.active_tournaments().contains_key(&tournament_id),
            "Tournament does not exist"
        );

        let tournament = match self.active_tournaments().get(&tournament_id) {
            Some(t) => t,
            None => sc_panic!("Tournament does not exist"),
        };
        let game_config = match self.registered_games().get(&tournament.game_id) {
            Some(g) => g,
            None => sc_panic!("Game config does not exist"),
        };

        require!(
            tournament.status == TournamentStatus::Completed,
            "Tournament not completed yet"
        );

        // Check if already claimed
        let claim_key = self.get_claim_key(&tournament_id, &caller);
        require!(
            !self.spectator_claims().contains_key(&claim_key),
            "Already claimed winnings"
        );

        let total_spectator_pool = self.spectator_pool_total(&tournament_id).get();
        require!(total_spectator_pool > 0, "No spectator pool");

        // Calculate house fee (basis points: 10,000 = 100.00%)
        let house_fee = total_spectator_pool.clone() * game_config.house_fee_percentage / 10_000u32;
        let remaining_pool = &total_spectator_pool - &house_fee;

        // Find which position the caller bet on (if any)
        let mut caller_winnings = BigUint::zero();

        for (position, winner) in tournament.final_podium.iter().enumerate() {
            let bets = self.spectator_bets(&tournament_id, &winner).get();

            // Calculate total bet on this winner
            let mut total_bet_on_winner = BigUint::zero();
            let mut caller_bet_amount = BigUint::zero();

            for bet in bets.iter() {
                total_bet_on_winner += &bet.amount;
                if bet.bettor_address == caller {
                    caller_bet_amount += &bet.amount;
                }
            }

            if caller_bet_amount > 0 && total_bet_on_winner > 0 {
                // Calculate this position's share of the pool
                let position_percentage = game_config.prize_distribution_percentages.get(position);
                let position_pool = &remaining_pool * position_percentage / 10_000u32;

                // Calculate caller's share
                let caller_share = &position_pool * &caller_bet_amount / &total_bet_on_winner;
                caller_winnings += caller_share;
            }
        }

        require!(caller_winnings > 0u32, "No winnings to claim");

        // Mark as claimed
        self.spectator_claims().insert(claim_key, true);

        // Send winnings
        self.send().direct_egld(&caller, &caller_winnings);
    }
}
