multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::models::{GameConfig, Tournament};

#[multiversx_sc::module]
pub trait HelperModule: crate::storage::StorageModule {
    fn verify_result_signature(
        &self,
        _tournament_id: &ManagedBuffer,
        _winner_podium: &ManagedVec<ManagedAddress>,
        _signed_result: &ManagedBuffer,
        _game_config: &GameConfig<Self::Api>,
    ) {
        // Placeholder for signature verification logic
        // In production, this would verify the signed_result against the signing_server_address
        // For now, we'll assume the signature is valid
    }

    fn distribute_player_prizes(
        &self,
        tournament: &Tournament<Self::Api>,
        game_config: &GameConfig<Self::Api>,
    ) {
        // Calculate house fee (basis points: 10,000 = 100.00%)
        let house_fee = &tournament.prize_pool * game_config.house_fee_percentage / 10_000u32;
        let remaining_pool = &tournament.prize_pool - &house_fee;

        // Send house fee to owner
        if house_fee > 0 {
            let owner = self.owner().get();
            self.send().direct_egld(&owner, &house_fee);
        }

        // Distribute prizes to winners
        for (position, winner) in tournament.final_podium.iter().enumerate() {
            let percentage = game_config.prize_distribution_percentages.get(position);
            let prize_amount = &remaining_pool * percentage / 10_000u32;

            if prize_amount > 0 {
                self.send().direct_egld(&winner, &prize_amount);
            }
        }
    }

    fn get_claim_key(
        &self,
        tournament_id: &ManagedBuffer,
        caller: &ManagedAddress,
    ) -> ManagedBuffer {
        let mut key = ManagedBuffer::new();
        key.append(tournament_id);
        key.append(&ManagedBuffer::from(b"_"));
        key.append(&caller.as_managed_buffer());
        key
    }

    fn require_owner(&self) {
        let caller = self.blockchain().get_caller();
        let owner = self.owner().get();
        require!(caller == owner, "Only owner can call this function");
    }
}
