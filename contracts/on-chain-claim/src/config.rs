use crate::address_info::*;
use crate::season_info::*;
use multiversx_sc::imports::*;

pub const MAX_REPAIR_GAP: u64 = 5;
pub const FIRST_SEASON_ID: u16 = 1;

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
        let current_season = self.get_current_season();
        if current_season.id == FIRST_SEASON_ID {
            let address_info = self.address_info(address);
            if address_info.is_empty() {
                return AddressInfo::default();
            }

            return address_info.get();
        }

        let address_info_by_season_mapper = self.address_info_by_season(address, current_season.id);

        if address_info_by_season_mapper.is_empty() {
            return AddressInfo::default();
        }

        address_info_by_season_mapper.get()
    }

    #[view(getAddressInfoBySeason)]
    fn get_address_info_by_season(&self, address: &ManagedAddress, season_id: u16) -> AddressInfo {
        if season_id == FIRST_SEASON_ID {
            let address_info = self.address_info(address);
            if address_info.is_empty() {
                return AddressInfo::default();
            }

            return address_info.get();
        }

        let address_info_by_season_mapper = self.address_info_by_season(address, season_id);

        if address_info_by_season_mapper.is_empty() {
            return AddressInfo::default();
        }

        address_info_by_season_mapper.get()
    }

    #[view(canBeRepaired)]
    fn can_be_repaired(&self, address: &ManagedAddress) -> bool {
        let address_info_mapper = self.address_info(address);
        if address_info_mapper.is_empty() {
            return false;
        }

        let address_info = self.get_address_info(address);
        if address_info.total_epochs_claimed == 0 {
            return false;
        }

        let current_epoch = self.blockchain().get_block_epoch();
        let missed_epochs = self.get_missed_epochs(current_epoch, address_info.last_epoch_claimed);

        missed_epochs > 0 && missed_epochs <= MAX_REPAIR_GAP
    }

    #[view(getCurrentSeason)]
    fn get_current_season(&self) -> SeasonInfo {
        let current_epoch = self.blockchain().get_block_epoch();
        let seasons = self.seasons().get();

        let mut current_season: SeasonInfo = SeasonInfo::new(1u16, 0u64);

        for season in seasons.iter() {
            if season.start_epoch <= current_epoch {
                current_season = season;
            }
        }

        current_season
    }

    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;

    #[storage_mapper("address_info_by_season")]
    fn address_info_by_season(
        &self,
        address: &ManagedAddress,
        season: u16,
    ) -> SingleValueMapper<AddressInfo>;

    #[view(getSeasons)]
    #[storage_mapper("seasons")]
    fn seasons(&self) -> SingleValueMapper<ManagedVec<SeasonInfo>>;

    #[view(getRepairStreakPayment)]
    #[storage_mapper("repair_streak_payment")]
    fn repair_streak_payment(&self) -> SingleValueMapper<EsdtTokenPayment>;
}
