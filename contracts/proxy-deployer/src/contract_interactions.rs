multiversx_sc::imports!();

use crate::address_to_id_mapper::AddressId;
use crate::config;

#[multiversx_sc::module]
pub trait ContractInteractionsModule: config::ConfigModule {
    #[endpoint(contractDeploy)]
    fn contract_deploy(
        &self,
        template_address_id: AddressId,
        args: MultiValueEncoded<ManagedBuffer>,
    ) -> ManagedAddress {
        let caller = self.blockchain().get_caller();

        let mut arguments = ManagedArgBuffer::new();
        for arg in args {
            arguments.push_arg(arg);
        }

        let opt_template_address = self.address_ids().get_address(template_address_id);
        let template_address = match opt_template_address {
            Some(template_address) => template_address,
            None => sc_panic!("Template not found"),
        };

        let (new_contract_address, _) = self.send_raw().deploy_from_source_contract(
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &template_address,
            CodeMetadata::DEFAULT,
            &arguments,
        );

        self.deployer_contract_addresses(&caller)
            .insert(new_contract_address.clone());
        self.deployers_list().insert(caller);

        new_contract_address
    }

    #[endpoint(contractUpgrade)]
    fn contract_upgrade(
        &self,
        contract_address: ManagedAddress,
        template_address_id: AddressId,
        args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        require!(
            self.deployer_contract_addresses(&caller)
                .contains(&contract_address),
            "Caller is not the deployer of the contract"
        );

        let mut arguments = ManagedArgBuffer::new();
        for arg in args {
            arguments.push_arg(arg);
        }

        let opt_template_address = self.address_ids().get_address(template_address_id);
        let template_address = match opt_template_address {
            Some(template_address) => template_address,
            None => sc_panic!("Template not found"),
        };

        self.send_raw().upgrade_from_source_contract(
            &contract_address,
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &template_address,
            CodeMetadata::DEFAULT,
            &arguments,
        );
    }

    #[endpoint(callContractEndpoint)]
    fn call_contract_endpoint(
        &self,
        contract_address: ManagedAddress,
        function_name: ManagedBuffer,
        args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        require!(
            self.deployer_contract_addresses(&caller)
                .contains(&contract_address),
            "Caller is not the deployer of the contract"
        );

        let gas_left = self.blockchain().get_gas_left();
        let mut contract_call = self
            .send()
            .contract_call::<()>(contract_address, function_name)
            .with_gas_limit(gas_left);

        for arg in args {
            contract_call.push_raw_argument(arg);
        }
        let _: IgnoreValue = contract_call.execute_on_dest_context();
    }
}
