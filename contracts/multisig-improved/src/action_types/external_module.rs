use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

multiversx_sc::imports!();

pub type ModuleId = AddressId;

mod external_module_proxy {
    use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait ExternalModuleProxy {
        #[view(canExecute)]
        fn can_execute(
            &self,
            sc_address: ManagedAddress,
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
    fn can_execute_action(
        &self,
        proposer: &ManagedAddress,
        sc_address: &ManagedAddress,
        egld_value: &BigUint,
        esdt_payments: &PaymentsVec<Self::Api>,
    ) -> bool {
        let module_id_mapper = self.module_id();
        let module_id = module_id_mapper.get_id(&sc_address);
        if module_id == NULL_ID {
            return false;
        }

        for module_id in self.active_modules_ids().iter() {
            let opt_module_address = module_id_mapper.get_address(module_id);
            require!(opt_module_address.is_some(), "Invalid setup");

            let module_address = unsafe { opt_module_address.unwrap_unchecked() };
            let can_execute: bool = self
                .external_sc_proxy(module_address)
                .can_execute(
                    sc_address.clone(),
                    proposer.clone(),
                    egld_value.clone(),
                    esdt_payments.clone(),
                )
                .execute_on_dest_context();

            if can_execute {
                return true;
            }
        }

        false
    }

    #[proxy]
    fn external_sc_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> external_module_proxy::Proxy<Self::Api>;

    #[storage_mapper("moduleId")]
    fn module_id(&self) -> AddressToIdMapper<Self::Api>;

    #[view(getNrDeployedModules)]
    #[storage_mapper("nrDeployModules")]
    fn nr_deployed_modules(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("activeModulesIds")]
    fn active_modules_ids(&self) -> UnorderedSetMapper<ModuleId>;
}
