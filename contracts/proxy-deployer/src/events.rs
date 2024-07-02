use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[derive(TypeAbi, TopEncode)]
pub struct DeployContractEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    template: ManagedAddress<M>,
    deployed_address: ManagedAddress<M>,
    arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(TypeAbi, TopEncode)]
pub struct UpgradeContractEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    template: ManagedAddress<M>,
    upgraded_address: ManagedAddress<M>,
    arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(TypeAbi, TopEncode)]
pub struct ContractCallEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    contract_address: ManagedAddress<M>,
    function: ManagedBuffer<M>,
    arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(TypeAbi, TopEncode)]
pub struct ChangeOwnerEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    contract_address: ManagedAddress<M>,
    old_owner: ManagedAddress<M>,
    new_owner: ManagedAddress<M>,
}

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_deploy_contract_event(
        &self,
        caller: ManagedAddress,
        template: ManagedAddress,
        deployed_address: ManagedAddress,
        arguments: ManagedVec<ManagedBuffer>,
    ) {
        let deploy_contract_event = DeployContractEvent {
            caller: caller.clone(),
            template,
            deployed_address,
            arguments,
        };

        self.deploy_contract_event(
            caller,
            self.blockchain().get_block_round(),
            deploy_contract_event,
        );
    }

    fn emit_upgrade_contract_event(
        &self,
        caller: ManagedAddress,
        template: ManagedAddress,
        upgraded_address: ManagedAddress,
        arguments: ManagedVec<ManagedBuffer>,
    ) {
        let upgrade_contract_event = UpgradeContractEvent {
            caller: caller.clone(),
            template,
            upgraded_address,
            arguments,
        };

        self.upgrade_contract_event(
            caller,
            self.blockchain().get_block_round(),
            upgrade_contract_event,
        );
    }

    fn emit_contract_call_event(
        &self,
        caller: ManagedAddress,
        contract_address: ManagedAddress,
        function: ManagedBuffer,
        arguments: ManagedVec<ManagedBuffer>,
    ) {
        let contract_call_event = ContractCallEvent {
            caller: caller.clone(),
            contract_address,
            function,
            arguments,
        };

        self.contract_call_event(
            caller,
            self.blockchain().get_block_round(),
            contract_call_event,
        );
    }

    fn emit_change_owner_event(
        &self,
        caller: ManagedAddress,
        contract_address: ManagedAddress,
        old_owner: ManagedAddress,
        new_owner: ManagedAddress,
    ) {
        let change_owner_event = ChangeOwnerEvent {
            caller: caller.clone(),
            contract_address,
            old_owner,
            new_owner,
        };

        self.change_owner_event(
            caller,
            self.blockchain().get_block_round(),
            change_owner_event,
        );
    }

    #[event("deploy_contract")]
    fn deploy_contract_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] current_round: u64,
        deploy_contract_event: DeployContractEvent<Self::Api>,
    );

    #[event("upgrade_contract")]
    fn upgrade_contract_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] current_round: u64,
        upgrade_contract_event: UpgradeContractEvent<Self::Api>,
    );

    #[event("contract_call")]
    fn contract_call_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] current_round: u64,
        contract_call_event: ContractCallEvent<Self::Api>,
    );

    #[event("change_owner")]
    fn change_owner_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] current_round: u64,
        change_owner_event: ChangeOwnerEvent<Self::Api>,
    );
}
