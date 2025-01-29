use multiversx_sc::imports::*;

use crate::events::{self};
use multiversx_sc::api::CHANGE_OWNER_BUILTIN_FUNC_NAME;
use multiversx_sc_modules::pause;

use crate::config::{self, OngoingUpgradeOperation};

#[multiversx_sc::module]
pub trait ContractInteractionsModule:
    config::ConfigModule + events::EventsModule + pause::PauseModule
{
    #[endpoint(contractDeploy)]
    fn contract_deploy(
        &self,
        template_address: ManagedAddress,
        args: MultiValueEncoded<ManagedBuffer>,
    ) -> ManagedAddress {
        self.can_call_endpoint(None);
        require!(
            self.blockchain().is_smart_contract(&template_address),
            "Template address is not a SC"
        );
        require!(
            self.templates_list().contains(&template_address),
            "Template address is not whitelisted"
        );

        let ongoing_upgrade_operation_mapper = self.ongoing_upgrade_operation();
        if !ongoing_upgrade_operation_mapper.is_empty() {
            let ongoing_upgrade_operation = ongoing_upgrade_operation_mapper.get();
            require!(
                ongoing_upgrade_operation.template_address != template_address,
                "There is an ongoing upgrade operation for this template address"
            );
        };

        let gas_left = self.blockchain().get_gas_left();
        let new_contract_address = self
            .tx()
            .raw_deploy()
            .arguments_raw(args.to_arg_buffer())
            .gas(gas_left)
            .from_source(template_address.clone())
            .code_metadata(self.blockchain().get_code_metadata(&template_address))
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        let caller = self.blockchain().get_caller();
        let owner = self.blockchain().get_owner_address();
        require!(caller != owner, "The owner cannot deploy contracts");

        self.deployer_contracts(&caller)
            .insert(new_contract_address.clone());
        self.deployed_contracts_list_by_template(&template_address)
            .update(|deployed_contracts| {
                deployed_contracts.push(new_contract_address.clone());
            });
        self.contract_template(&new_contract_address)
            .set(&template_address);
        let mut deployed_addresses = match self
            .deployer_template_addresses(&caller)
            .get(&template_address)
        {
            Some(addresses) => addresses,
            None => ManagedVec::new(),
        };
        deployed_addresses.push(new_contract_address.clone());
        self.deployer_template_addresses(&caller)
            .insert(template_address.clone(), deployed_addresses);
        self.deployers_list().insert(caller.clone());

        self.emit_deploy_contract_event(
            caller,
            template_address,
            new_contract_address.clone(),
            args.into_vec_of_buffers(),
        );

        new_contract_address
    }

    #[endpoint(contractUpgrade)]
    fn contract_upgrade(
        &self,
        contract_address: ManagedAddress,
        args: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.can_call_endpoint(Some(contract_address.clone()));
        require!(
            self.blockchain().is_smart_contract(&contract_address),
            "Contract address is not a SC"
        );
        let contract_template_mapper = self.contract_template(&contract_address);
        require!(!contract_template_mapper.is_empty(), "No template found");
        let template_address = contract_template_mapper.get();

        self.tx()
            .to(contract_address.clone())
            .egld(BigUint::zero())
            .gas(self.blockchain().get_gas_left())
            .raw_upgrade()
            .from_source(template_address.clone())
            .code_metadata(self.blockchain().get_code_metadata(&contract_address))
            .arguments_raw(args.to_arg_buffer())
            .upgrade_async_call_and_exit();

        self.emit_upgrade_contract_event(
            self.blockchain().get_caller(),
            template_address,
            contract_address,
            args.into_vec_of_buffers(),
        );
    }

    #[endpoint(contractCallByAddress)]
    fn contract_call_by_address(
        &self,
        contract_address: ManagedAddress,
        function_name: ManagedBuffer,
        args: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.can_call_endpoint(Some(contract_address.clone()));
        require!(
            self.blockchain().is_smart_contract(&contract_address),
            "Contract address is not a SC"
        );
        require!(
            function_name != ManagedBuffer::from(CHANGE_OWNER_BUILTIN_FUNC_NAME),
            "Use the dedicated change owner endpoint instead"
        );

        self.tx()
            .to(contract_address.clone())
            .raw_call(function_name.clone())
            .gas(self.blockchain().get_gas_left())
            .arguments_raw(args.to_arg_buffer())
            .sync_call();

        self.emit_contract_call_event(
            self.blockchain().get_caller(),
            contract_address,
            function_name,
            args.into_vec_of_buffers(),
        );
    }

    /// Use this endpoint to transfer the ownership
    /// This is needed to properly update the stored data
    #[endpoint(changeOwnerAddress)]
    fn change_owner(
        &self,
        contract_address: ManagedAddress,
        new_owner: ManagedAddress,
        opt_orig_owner: OptionalValue<ManagedAddress>,
    ) {
        self.can_call_endpoint(Some(contract_address.clone()));
        require!(
            self.blockchain().is_smart_contract(&contract_address),
            "Contract address is not a SC"
        );
        require!(
            !self.blacklisted_deployers_list().contains(&new_owner),
            "New owner is blacklisted"
        );
        let mut caller = self.blockchain().get_caller();
        if caller == self.blockchain().get_owner_address() {
            require!(opt_orig_owner.is_some(), "Must provide original owner");
            caller = unsafe { opt_orig_owner.into_option().unwrap_unchecked() };
        }
        let contract_template_mapper = self.contract_template(&contract_address);
        require!(!contract_template_mapper.is_empty(), "No template found");

        let mut contract_processed = false;
        let template_address = contract_template_mapper.take();
        let deployer_template_addresses_mapper = self.deployer_template_addresses(&caller);
        let mut deployer_template_addresses =
            match deployer_template_addresses_mapper.get(&template_address) {
                Some(addresses) => addresses,
                None => sc_panic!("No mapped deployer template found"),
            };

        for index in 0..deployer_template_addresses.len() {
            let deployed_address = deployer_template_addresses.get(index).clone();
            if deployed_address == contract_address {
                deployer_template_addresses.remove(index);
                self.deployer_template_addresses(&caller)
                    .insert(template_address, deployer_template_addresses);

                contract_processed = true;
                break;
            }
        }

        require!(contract_processed, "Contract not found for deployer");
        self.deployer_contracts(&caller)
            .swap_remove(&contract_address);
        if self.deployer_contracts(&caller).is_empty() {
            self.deployers_list().swap_remove(&caller);
        }

        let () = self
            .send()
            .change_owner_address(contract_address.clone(), &new_owner)
            .sync_call();

        self.emit_change_owner_event(
            self.blockchain().get_caller(),
            contract_address,
            caller,
            new_owner,
        );
    }

    /// Allows the owner to bulk upgrade all the contracts by starting an ongoing upgrade operation
    /// The first time when the endpoint is called, the optional arguments are required
    /// After that the endpoint needs to be called without the optional args, until the upgrade operation is finished
    #[only_owner]
    #[allow_multiple_var_args]
    #[endpoint(upgradeContractsByTemplate)]
    fn upgrade_contracts_by_template(
        &self,
        gas_per_action: u64,
        opt_template_address: OptionalValue<ManagedAddress>,
        args: MultiValueEncoded<ManagedBuffer>,
    ) -> bool {
        let mut ongoing_upgrade_operation = self.get_ongoing_operation(opt_template_address, args);

        let default_gas_for_save = self.default_gas_for_save_operation().get();
        while self.blockchain().get_gas_left() >= gas_per_action + default_gas_for_save
            && !ongoing_upgrade_operation.contracts_remaining.is_empty()
        {
            let contract_address = ongoing_upgrade_operation.contracts_remaining.get(0).clone();
            // If the contract_template storage is empty, it means the contracts ownership was transfered
            if !self.contract_template(&contract_address).is_empty() {
                self.tx()
                    .to(contract_address.clone())
                    .egld(BigUint::zero())
                    .gas(gas_per_action)
                    .raw_upgrade()
                    .from_source(ongoing_upgrade_operation.template_address.clone())
                    .code_metadata(self.blockchain().get_code_metadata(&contract_address))
                    .arguments_raw(ongoing_upgrade_operation.arguments.clone())
                    .upgrade_async_call_and_exit();
                ongoing_upgrade_operation
                    .processed_contracts
                    .push(contract_address);
            }
            ongoing_upgrade_operation.contracts_remaining.remove(0);
        }
        if !ongoing_upgrade_operation.contracts_remaining.is_empty() {
            self.ongoing_upgrade_operation()
                .set(ongoing_upgrade_operation);
            return false;
        }

        self.deployed_contracts_list_by_template(&ongoing_upgrade_operation.template_address)
            .set(ongoing_upgrade_operation.processed_contracts);
        self.ongoing_upgrade_operation().clear();
        true
    }

    #[only_owner]
    #[endpoint(clearOngoingUpgradeOperation)]
    fn clear_ongoing_upgrade_operation(&self) {
        self.ongoing_upgrade_operation().clear();
    }

    fn get_ongoing_operation(
        &self,
        opt_template_address: OptionalValue<ManagedAddress>,
        args: MultiValueEncoded<ManagedBuffer>,
    ) -> OngoingUpgradeOperation<Self::Api> {
        let ongoing_operation_mapper = self.ongoing_upgrade_operation();
        if opt_template_address.is_none() {
            require!(
                !ongoing_operation_mapper.is_empty(),
                "There is no operation ongoing"
            );
            return ongoing_operation_mapper.get();
        }

        require!(
            ongoing_operation_mapper.is_empty(),
            "Another operation is currently ongoing"
        );
        let template_address = opt_template_address
            .into_option()
            .unwrap_or_else(|| sc_panic!("Error decoding the template address"));
        require!(
            self.blockchain().is_smart_contract(&template_address),
            "Template address is not a SC"
        );
        let contracts_by_template = self
            .deployed_contracts_list_by_template(&template_address)
            .get();
        require!(
            !contracts_by_template.is_empty(),
            "No contracts deployed with this template"
        );

        OngoingUpgradeOperation::new(
            template_address,
            args.to_arg_buffer(),
            contracts_by_template,
            ManagedVec::new(),
        )
    }

    fn can_call_endpoint(&self, opt_contract_address: Option<ManagedAddress>) {
        let caller = self.blockchain().get_caller();
        let owner = self.blockchain().get_owner_address();

        if caller == owner {
            return;
        }

        self.require_not_paused();
        require!(
            !self.blacklisted_deployers_list().contains(&caller),
            "User is blacklisted"
        );

        if opt_contract_address.is_none() {
            return;
        }
        let contract_address =
            opt_contract_address.unwrap_or_else(|| sc_panic!("Cannot unwrap the contract address"));
        require!(
            self.deployer_contracts(&caller).contains(&contract_address),
            "Only the deployer can call this function"
        );
    }
}
