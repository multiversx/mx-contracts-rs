use crate::common_types::{
    action::{Action, ActionId, CallActionData, DeployArgs, EsdtTransferExecuteData, GasLimit},
    user_role::{change_user_role, UserRole},
};

use crate::ms_endpoints::callbacks::CallbackProxy as _;

use super::external_module::DISABLED;

multiversx_sc::imports!();

/// Gas required to finish transaction after transfer-execute.
const PERFORM_ACTION_FINISH_GAS: u64 = 300_000;
pub const MAX_BOARD_MEMBERS: usize = 30;

#[multiversx_sc::module]
pub trait ExecuteActionModule:
    crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
    + super::external_module::ExternalModuleModule
    + crate::external::events::EventsModule
    + crate::ms_endpoints::callbacks::CallbacksModule
{
    fn try_execute_deploy(
        &self,
        action_id: ActionId,
        action: &Action<Self::Api>,
    ) -> OptionalValue<ManagedAddress> {
        if let Action::SCDeployFromSource(args) = action {
            let new_address = self.deploy_from_source(action_id, args.clone());

            return OptionalValue::Some(new_address);
        }
        if let Action::DeployModuleFromSource(args) = action {
            let new_address = self.deploy_from_source(action_id, args.clone());
            let module_id = self.module_id().insert_new(&new_address);
            let proposer_id = self.deploy_module_proposer(action_id).take();
            self.module_owner(module_id).set(proposer_id);

            return OptionalValue::Some(new_address);
        }

        OptionalValue::None
    }

    fn execute_action_by_type(&self, action_id: ActionId, action: Action<Self::Api>) {
        match action {
            Action::Nothing => {}
            Action::AddBoardMember(board_member_address) => {
                self.add_board_member(action_id, board_member_address);
            }
            Action::AddProposer(proposer_address) => {
                self.add_proposer(action_id, proposer_address);
            }
            Action::RemoveUser(user_address) => {
                self.remove_user(action_id, user_address);
            }
            Action::ChangeQuorum(new_quorum) => {
                self.change_quorum(action_id, new_quorum);
            }
            Action::SendTransferExecuteEgld(call_data) => {
                self.send_transfer_execute_egld(action_id, call_data);
            }
            Action::SendTransferExecuteEsdt(call_data) => {
                self.send_transfer_execute_esdt(action_id, call_data);
            }
            Action::SendAsyncCall(call_data) => {
                self.send_async_call(action_id, call_data);
            }
            Action::SCUpgradeFromSource { sc_address, args } => {
                self.upgrade_from_source(action_id, sc_address, args);
            }
            Action::UpgradeModuleFromSource { sc_address, args } => {
                let module_id = self.module_id().get_id_non_zero(&sc_address);
                self.module_status(module_id).set(DISABLED);

                self.upgrade_from_source(action_id, sc_address, args);
            }
            _ => {} // Deploy cases handled in another function
        };
    }

    fn add_board_member(&self, action_id: ActionId, board_member_address: ManagedAddress) {
        require!(
            self.num_board_members().get() < MAX_BOARD_MEMBERS,
            "board size cannot exceed limit"
        );

        change_user_role(self, action_id, board_member_address, UserRole::BoardMember);
    }

    fn add_proposer(&self, action_id: ActionId, proposer_address: ManagedAddress) {
        change_user_role(self, action_id, proposer_address, UserRole::Proposer);

        // validation required for the scenario when a board member becomes a proposer
        let quorum = self.quorum().get();
        let board_members = self.num_board_members().get();
        self.require_valid_quorum(quorum, board_members);
    }

    fn remove_user(&self, action_id: ActionId, user_address: ManagedAddress) {
        change_user_role(self, action_id, user_address, UserRole::None);

        let num_board_members = self.num_board_members().get();
        let num_proposers = self.num_proposers().get();
        require!(
            num_board_members + num_proposers > 0,
            "cannot remove all board members and proposers"
        );

        let quorum = self.quorum().get();
        self.require_valid_quorum(quorum, num_board_members);
    }

    fn change_quorum(&self, action_id: ActionId, new_quorum: usize) {
        let board_members = self.num_board_members().get();
        self.require_valid_quorum(new_quorum, board_members);

        self.quorum().set(new_quorum);
        self.perform_change_quorum_event(action_id, new_quorum);
    }

    fn send_transfer_execute_egld(
        &self,
        action_id: ActionId,
        call_data: CallActionData<Self::Api>,
    ) {
        let gas = call_data
            .opt_gas_limit
            .unwrap_or_else(|| self.ensure_and_get_gas_for_transfer_exec());

        self.perform_transfer_execute_egld_event(
            action_id,
            &call_data.to,
            &call_data.egld_amount,
            gas,
            &call_data.endpoint_name,
            call_data.arguments.as_multi(),
        );

        let result = self.send_raw().direct_egld_execute(
            &call_data.to,
            &call_data.egld_amount,
            gas,
            &call_data.endpoint_name,
            &call_data.arguments.into(),
        );
        if let Result::Err(e) = result {
            sc_panic!(e);
        }
    }

    fn send_transfer_execute_esdt(
        &self,
        action_id: ActionId,
        call_data: EsdtTransferExecuteData<Self::Api>,
    ) {
        let gas = call_data
            .opt_gas_limit
            .unwrap_or_else(|| self.ensure_and_get_gas_for_transfer_exec());

        self.perform_transfer_execute_esdt_event(
            action_id,
            &call_data.to,
            &call_data.tokens,
            gas,
            &call_data.endpoint_name,
            call_data.arguments.as_multi(),
        );

        let result = self.send_raw().multi_esdt_transfer_execute(
            &call_data.to,
            &call_data.tokens,
            gas,
            &call_data.endpoint_name,
            &call_data.arguments.into(),
        );
        if let Result::Err(e) = result {
            sc_panic!(e);
        }
    }

    fn send_async_call(&self, action_id: ActionId, call_data: CallActionData<Self::Api>) {
        let gas = call_data
            .opt_gas_limit
            .unwrap_or_else(|| self.ensure_and_get_gas_for_transfer_exec());
        self.perform_async_call_event(
            action_id,
            &call_data.to,
            &call_data.egld_amount,
            gas,
            &call_data.endpoint_name,
            call_data.arguments.as_multi(),
        );
        self.send()
            .contract_call::<()>(call_data.to, call_data.endpoint_name)
            .with_egld_transfer(call_data.egld_amount)
            .with_raw_arguments(call_data.arguments.into())
            .with_gas_limit(gas)
            .async_call()
            .with_callback(self.callbacks().perform_async_call_callback())
            .call_and_exit();
    }

    fn deploy_from_source(
        &self,
        action_id: ActionId,
        args: DeployArgs<Self::Api>,
    ) -> ManagedAddress {
        let gas_left = self.blockchain().get_gas_left();
        self.perform_deploy_from_source_event(
            action_id,
            &args.amount,
            &args.source,
            args.code_metadata,
            gas_left,
            args.arguments.as_multi(),
        );
        let (new_address, _) = self.send_raw().deploy_from_source_contract(
            gas_left,
            &args.amount,
            &args.source,
            args.code_metadata,
            &args.arguments.into(),
        );

        new_address
    }

    fn upgrade_from_source(
        &self,
        action_id: ActionId,
        sc_address: ManagedAddress,
        args: DeployArgs<Self::Api>,
    ) {
        let gas_left = self.blockchain().get_gas_left();
        self.perform_upgrade_from_source_event(
            action_id,
            &sc_address,
            &args.amount,
            &args.source,
            args.code_metadata,
            gas_left,
            args.arguments.as_multi(),
        );
        self.send_raw().upgrade_from_source_contract(
            &sc_address,
            gas_left,
            &args.amount,
            &args.source,
            args.code_metadata,
            &args.arguments.into(),
        );
    }

    fn clear_action(&self, action_id: ActionId) {
        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signer_ids(action_id).clear();

        let group_id = self.group_for_action(action_id).take();
        if group_id != 0 {
            let _ = self.action_groups(group_id).swap_remove(&action_id);
        }
    }

    fn ensure_and_get_gas_for_transfer_exec(&self) -> GasLimit {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > PERFORM_ACTION_FINISH_GAS,
            "insufficient gas for call"
        );

        gas_left - PERFORM_ACTION_FINISH_GAS
    }

    fn require_valid_quorum(&self, quorum: usize, num_board_members: usize) {
        require!(
            quorum <= num_board_members,
            "quorum cannot exceed board size"
        );
    }
}
