multiversx_sc::imports!();

use multiversx_sc_modules::pause;

use crate::config::{self, OngoingUpgradeOperation};

#[multiversx_sc::module]
pub trait ContractInteractionsModule: config::ConfigModule + pause::PauseModule {
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

        let (new_contract_address, _) = self.send_raw().deploy_from_source_contract(
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &template_address,
            self.blockchain().get_code_metadata(&template_address),
            &args.to_arg_buffer(),
        );

        let caller = self.blockchain().get_caller();
        let mut deployed_addresses = match self
            .deployer_template_addresses(&caller)
            .get(&template_address)
        {
            Some(addresses) => addresses,
            None => ManagedVec::new(),
        };
        deployed_addresses.push(new_contract_address.clone());

        self.deployer_contracts(&caller).add(&new_contract_address);
        self.deployed_contracts_list_by_template(&template_address)
            .update(|deployed_contracts| {
                deployed_contracts.push(new_contract_address.clone());
            });
        self.deployer_template_addresses(&caller)
            .insert(template_address, deployed_addresses);
        self.deployers_list().insert(caller);

        new_contract_address
    }

    #[endpoint(contractUpgrade)]
    fn contract_upgrade(
        &self,
        contract_address: ManagedAddress,
        template_address: ManagedAddress,
        args: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.can_call_endpoint(Some(contract_address.clone()));
        require!(
            self.blockchain().is_smart_contract(&contract_address),
            "Contract address is not a SC"
        );
        require!(
            self.blockchain().is_smart_contract(&template_address),
            "Template address is not a SC"
        );

        self.send_raw().upgrade_from_source_contract(
            &contract_address,
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &template_address,
            self.blockchain().get_code_metadata(&template_address),
            &args.to_arg_buffer(),
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

        self.send()
            .contract_call::<()>(contract_address, function_name)
            .with_gas_limit(self.blockchain().get_gas_left())
            .with_raw_arguments(args.to_arg_buffer())
            .execute_on_dest_context()
    }

    #[only_owner]
    #[allow_multiple_var_args]
    #[endpoint(upgradeContractsByTemplate)]
    fn upgrade_contracts_by_template(
        &self,
        gas_per_action: u64,
        opt_template_address: OptionalValue<ManagedAddress>,
        opt_args: OptionalValue<MultiValueEncoded<ManagedBuffer>>,
    ) -> bool {
        let mut ongoing_upgrade_operation =
            self.get_ongoing_operation(opt_template_address, opt_args);

        let default_gas_for_save = self.default_gas_for_save_operation().get();
        while self.blockchain().get_gas_left() >= gas_per_action + default_gas_for_save
            && !ongoing_upgrade_operation.contracts_remaining.is_empty()
        {
            let contract_address = ongoing_upgrade_operation
                .contracts_remaining
                .get(0)
                .clone_value();
            self.send_raw().upgrade_from_source_contract(
                &contract_address,
                gas_per_action,
                &BigUint::zero(),
                &ongoing_upgrade_operation.template_address,
                CodeMetadata::DEFAULT,
                &ongoing_upgrade_operation.arguments,
            );
            ongoing_upgrade_operation.contracts_remaining.remove(0);
        }
        if !ongoing_upgrade_operation.contracts_remaining.is_empty() {
            self.ongoing_upgrade_operation()
                .set(ongoing_upgrade_operation);
            return false;
        }

        self.ongoing_upgrade_operation().clear();
        true
    }

    fn get_ongoing_operation(
        &self,
        opt_template_address: OptionalValue<ManagedAddress>,
        opt_args: OptionalValue<MultiValueEncoded<ManagedBuffer>>,
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
        let args = match opt_args.into_option() {
            Some(args) => args,
            None => MultiValueEncoded::new(),
        };

        OngoingUpgradeOperation::new(
            template_address,
            args.to_arg_buffer(),
            contracts_by_template,
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
