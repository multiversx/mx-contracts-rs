use crate::address_info::*;
use multiversx_sc::imports::*;

pub const MAX_REPAIR_GAP: u64 = 5;
pub const FIRST_SEASON_ID: usize = 1;
pub const FIRST_SEASON_START_EPOCH: u64 = 1400;

type Epoch = u64;

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_same_shard(&self, address: &ManagedAddress) {
        let address_shard = self.blockchain().get_shard_of_address(address);
        let sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&sc_address);

        require!(address_shard == sc_shard, "wrong shard");
    }

    fn get_missed_epochs(&self, current_epoch: u64, last_epoch_claimed: u64) -> u64 {
        if current_epoch <= last_epoch_claimed {
            return 0;
        }

        current_epoch - last_epoch_claimed - 1
    }

    #[view(getAddressInfo)]
    fn get_address_info(&self, address: &ManagedAddress) -> AddressInfo {
        let current_season_id: usize = self.get_current_season();
        let address_info_by_season_mapper = self.address_info_by_season(address, current_season_id);

        if !address_info_by_season_mapper.is_empty() {
            return address_info_by_season_mapper.get();
        }

        let address_info_mapper = self.address_info(address);
        if current_season_id == FIRST_SEASON_ID && !address_info_mapper.is_empty() {
            return address_info_mapper.get();
        }

        AddressInfo::default()
    }

    #[view(getAddressInfoBySeason)]
    fn get_address_info_by_season(
        &self,
        address: &ManagedAddress,
        season_id: usize,
    ) -> AddressInfo {
        let address_info_by_season_mapper = self.address_info_by_season(address, season_id);

        if !address_info_by_season_mapper.is_empty() {
            return address_info_by_season_mapper.get();
        }

        let address_info_mapper = self.address_info(address);
        if season_id == FIRST_SEASON_ID && !address_info_mapper.is_empty() {
            return address_info_mapper.get();
        }

        AddressInfo::default()
    }

    #[view(canBeRepaired)]
    fn can_be_repaired(&self, address: &ManagedAddress) -> bool {
        let address_info = self.get_address_info(address);
        if address_info.total_epochs_claimed == 0 {
            return false;
        }

        let current_epoch = self.blockchain().get_block_epoch();
        let missed_epochs = self.get_missed_epochs(current_epoch, address_info.last_epoch_claimed);

        missed_epochs > 0 && missed_epochs <= MAX_REPAIR_GAP
    }

    #[view(getCurrentSeason)]
    fn get_current_season(&self) -> usize {
        let current_epoch = self.blockchain().get_block_epoch();
        let seasons = self.seasons();

        let last_season_starting_epoch = seasons.get(seasons.len());
        if last_season_starting_epoch <= current_epoch {
            return seasons.len();
        }

        seasons.len() - 1
    }

    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;

    #[storage_mapper("address_info_by_season")]
    fn address_info_by_season(
        &self,
        address: &ManagedAddress,
        season_id: usize,
    ) -> SingleValueMapper<AddressInfo>;

    /**
     * This parameter is referring to the start epoch of the season.
     */
    #[view(getSeasons)]
    #[storage_mapper("seasons")]
    fn seasons(&self) -> VecMapper<Epoch>;

    #[view(getRepairStreakPayment)]
    #[storage_mapper("repair_streak_payment")]
    fn repair_streak_payment(&self) -> SingleValueMapper<EsdtTokenPayment>;
}
