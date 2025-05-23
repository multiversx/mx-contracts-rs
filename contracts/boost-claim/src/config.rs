use multiversx_sc::imports::*;

use crate::address_boost_info::*;

pub type Timestamp = u64;

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_same_shard(&self, address: &ManagedAddress) {
        let address_shard = self.blockchain().get_shard_of_address(address);
        let sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&sc_address);

        require!(address_shard == sc_shard, "wrong shard");
    }

    #[view(getAddressBoostInfo)]
    fn get_address_boost_info(&self, address: &ManagedAddress) -> AddressBoostInfo {
        let mapper = self.address_boost_info(address);
        if mapper.is_empty() {
            return AddressBoostInfo::new(1, 0, 0);
        }

        mapper.get()
    }

    #[storage_mapper("addressBoostInfo")]
    fn address_boost_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressBoostInfo>;

    #[view(getLevelsPrizes)]
    #[storage_mapper("levelsPrizes")]
    fn levels_prizes(&self) -> VecMapper<ManagedBuffer>;

    #[view(getTimeDifferenceInSeconds)]
    #[storage_mapper("timeDifference")]
    fn time_difference_in_seconds(&self) -> SingleValueMapper<Timestamp>;

    #[event("boost_claim")]
    fn boost_claim_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] prize: &ManagedBuffer,
    );
}
