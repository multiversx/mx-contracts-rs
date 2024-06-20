use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::common_types::action::ActionId;

multiversx_sc::imports!();

pub type ModuleId = AddressId;

pub type ModuleStatus = bool;
pub const ENABLED: ModuleStatus = true;
pub const DISABLED: ModuleStatus = false;

mod external_module_proxy {
    use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait ExternalModuleProxy {
        #[view(canExecute)]
        fn can_execute(
            &self,
            original_caller: ManagedAddress,
            egld_value: BigUint,
            esdt_payments: PaymentsVec<Self::Api>,
        ) -> bool;
    }
}

#[multiversx_sc::module]
pub trait ExternalModuleModule:
    crate::common_functions::CommonFunctionsModule + crate::state::StateModule
{
    #[endpoint(enableModule)]
    fn enable_module(&self, module: ManagedAddress) {
        self.set_module_status_common(module, ENABLED);
    }

    #[endpoint(disableModule)]
    fn disable_module(&self, module: ManagedAddress) {
        self.set_module_status_common(module, DISABLED);
    }

    #[endpoint(addAdditionalAllowedAddresses)]
    fn add_additional_allowed_addresses(
        &self,
        module: ManagedAddress,
        addresses: MultiValueEncoded<ManagedAddress>,
    ) {
        let module_id = self.module_id().get_id_non_zero(&module);
        self.require_module_owner_caller(module_id);

        let mut mapper = self.additional_allowed_addresses(module_id);
        for addr in addresses {
            let (user_id, user_role) = self.get_id_and_role(&addr);
            user_role.require_can_propose::<Self::Api>();

            let _ = mapper.insert(user_id);
        }
    }

    #[endpoint(removeAdditionalAllowedAddresses)]
    fn remove_additional_allowed_addresses(
        &self,
        module: ManagedAddress,
        addresses: MultiValueEncoded<ManagedAddress>,
    ) {
        let module_id = self.module_id().get_id_non_zero(&module);
        self.require_module_owner_caller(module_id);

        let mut mapper = self.additional_allowed_addresses(module_id);
        for addr in addresses {
            let user_id = self.user_ids().get_id_non_zero(&addr);
            let removed = mapper.swap_remove(&user_id);
            require!(removed, "Address not in list");
        }
    }

    #[view(getModuleStatus)]
    fn get_module_status(&self, module: ManagedAddress) -> ModuleStatus {
        let module_id = self.module_id().get_id_non_zero(&module);

        self.module_status(module_id).get()
    }

    fn set_module_status_common(&self, module: ManagedAddress, status: ModuleStatus) {
        let module_id = self.module_id().get_id_non_zero(&module);
        self.require_module_owner_caller(module_id);

        self.module_status(module_id).set(status);
    }

    fn require_module_owner_caller(&self, module_id: ModuleId) {
        let caller = self.blockchain().get_caller();
        let module_owner = self.module_owner(module_id).get();
        let (caller_id, caller_role) = self.get_id_and_role(&caller);
        caller_role.require_can_propose::<Self::Api>();

        require!(
            caller_id == module_owner,
            "Only module owner can call this function"
        );
    }

    fn can_execute_action(
        &self,
        sc_address: ManagedAddress,
        egld_value: BigUint,
        esdt_payments: PaymentsVec<Self::Api>,
    ) -> bool {
        let module_id = self.module_id().get_id(&sc_address);
        if module_id == NULL_ID {
            return false;
        }
        if self.module_status(module_id).get() == DISABLED {
            return false;
        }

        let original_caller = self.blockchain().get_caller();
        let caller_id = self.user_ids().get_id_non_zero(&original_caller);
        let module_owner = self.module_owner(module_id).get();
        if caller_id != module_owner
            && !self
                .additional_allowed_addresses(module_id)
                .contains(&caller_id)
        {
            return false;
        }

        self.external_sc_proxy(sc_address)
            .can_execute(original_caller, egld_value, esdt_payments)
            .execute_on_dest_context()
    }

    #[proxy]
    fn external_sc_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> external_module_proxy::Proxy<Self::Api>;

    #[storage_mapper("moduleId")]
    fn module_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("moduleStatus")]
    fn module_status(&self, module_id: ModuleId) -> SingleValueMapper<ModuleStatus>;

    #[storage_mapper("deployModProposer")]
    fn deploy_module_proposer(&self, action_id: ActionId) -> SingleValueMapper<AddressId>;

    #[storage_mapper("moduleOwner")]
    fn module_owner(&self, module_id: ModuleId) -> SingleValueMapper<AddressId>;

    #[storage_mapper("addAllAddr")]
    fn additional_allowed_addresses(&self, module_id: ModuleId) -> UnorderedSetMapper<AddressId>;
}
