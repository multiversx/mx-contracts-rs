multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct OngoingUpgradeOperation<M: ManagedTypeApi> {
    pub template_address: ManagedAddress<M>,
    pub arguments: ManagedArgBuffer<M>,
    pub contracts_remaining: ManagedVec<M, ManagedAddress<M>>,
}

impl<M: ManagedTypeApi> OngoingUpgradeOperation<M> {
    #[inline]
    pub fn new(
        template_address: ManagedAddress<M>,
        arguments: ManagedArgBuffer<M>,
        contracts_remaining: ManagedVec<M, ManagedAddress<M>>,
    ) -> Self {
        OngoingUpgradeOperation {
            template_address,
            arguments,
            contracts_remaining,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[only_owner]
    #[endpoint(addDeployerToBlacklist)]
    fn add_deployer_to_blacklist(&self, blacklisted_address: ManagedAddress) {
        require!(
            self.deployers_list().contains(&blacklisted_address),
            "The address is not a deployer"
        );
        require!(
            !self
                .blacklisted_deployers_list()
                .contains(&blacklisted_address),
            "Address already blacklisted"
        );
        self.blacklisted_deployers_list()
            .insert(blacklisted_address);
    }

    #[only_owner]
    #[endpoint(removeDeployerFromBlacklist)]
    fn remove_deployer_from_blacklist(&self, address: ManagedAddress) {
        require!(
            self.blacklisted_deployers_list().contains(&address),
            "Address is not blacklisted"
        );

        self.blacklisted_deployers_list().swap_remove(&address);
    }

    #[only_owner]
    #[endpoint(setDefaultGasForSaveOperation)]
    fn set_default_gas_for_save_operation(&self, default_gas_for_save_operation: u64) {
        self.default_gas_for_save_operation()
            .set(default_gas_for_save_operation);
    }

    #[view(getAllDeployedContractsByTemplate)]
    #[storage_mapper("deployedContractsByTemplate")]
    fn deployed_contracts_list_by_template(
        &self,
        template_address: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

    #[view(getOngoingUpgradeOperations)]
    #[storage_mapper("ongoingUpgradeOperation")]
    fn ongoing_upgrade_operation(&self) -> SingleValueMapper<OngoingUpgradeOperation<Self::Api>>;

    #[view(getDefaultGasForSaveOperation)]
    #[storage_mapper("defaultGasForSaveOperation")]
    fn default_gas_for_save_operation(&self) -> SingleValueMapper<u64>;

    #[view(getAllDeployers)]
    #[storage_mapper("deployersList")]
    fn deployers_list(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getDeployerContractAddresses)]
    #[storage_mapper("deployerContractAddresses")]
    fn deployer_contract_addresses(
        &self,
        deployer_address: &ManagedAddress,
    ) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getAllBlacklistedDeployers)]
    #[storage_mapper("blacklistedDeployersList")]
    fn blacklisted_deployers_list(&self) -> UnorderedSetMapper<ManagedAddress>;
}
