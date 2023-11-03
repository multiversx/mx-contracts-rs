multiversx_sc::imports!();

use crate::address_to_id_mapper::{AddressId, AddressToIdMapper};

#[multiversx_sc::module]
pub trait ConfigModule {
    #[only_owner]
    #[endpoint(addContractTemplate)]
    fn add_contract_template(&self, template_address: ManagedAddress) -> AddressId {
        require!(
            self.blockchain().is_smart_contract(&template_address),
            "Invalid template address"
        );

        self.address_ids().insert_new(&template_address)
    }

    #[only_owner]
    #[endpoint(removeContractTemplate)]
    fn remove_contract_template(&self, address_id: AddressId) {
        require!(
            self.address_ids().contains_id(address_id),
            "Invalid address id"
        );

        self.address_ids().remove_by_id(address_id);
    }

    #[storage_mapper("addressIds")]
    fn address_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getAllDeployers)]
    #[storage_mapper("deployersList")]
    fn deployers_list(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getDeployerContractAddresses)]
    #[storage_mapper("deployerContractAddresses")]
    fn deployer_contract_addresses(
        &self,
        deployer_address: &ManagedAddress,
    ) -> UnorderedSetMapper<ManagedAddress>;
}
