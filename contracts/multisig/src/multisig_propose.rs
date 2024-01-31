use crate::{
    action::{Action, CallActionData, GasLimit},
    multisig_state::GroupId,
};

multiversx_sc::imports!();

/// Contains all events that can be emitted by the contract.
#[multiversx_sc::module]
pub trait MultisigProposeModule: crate::multisig_state::MultisigStateModule {
    fn propose_action(&self, action: Action<Self::Api>, opt_group_id: Option<GroupId>) -> usize {
        let (caller_id, caller_role) = self.get_caller_id_and_role();
        require!(
            caller_role.can_propose(),
            "only board members and proposers can propose"
        );

        let action_id = self.action_mapper().push(&action);
        if caller_role.can_sign() {
            // also sign
            // since the action is newly created, the caller can be the only signer
            self.action_signer_ids(action_id).insert(caller_id);
        }

        if let Some(group_id) = opt_group_id {
            require!(group_id != 0, "May not use group ID 0");

            let _ = self.action_groups(group_id).insert(action_id);
            self.group_for_action(action_id).set(group_id);
        }

        action_id
    }

    /// Initiates board member addition process.
    /// Can also be used to promote a proposer to board member.
    #[endpoint(proposeAddBoardMember)]
    fn propose_add_board_member(
        &self,
        board_member_address: ManagedAddress,
        opt_group_id: OptionalValue<GroupId>,
    ) -> usize {
        self.propose_action(
            Action::AddBoardMember(board_member_address),
            opt_group_id.into_option(),
        )
    }

    /// Initiates proposer addition process..
    /// Can also be used to demote a board member to proposer.
    #[endpoint(proposeAddProposer)]
    fn propose_add_proposer(
        &self,
        proposer_address: ManagedAddress,
        opt_group_id: OptionalValue<GroupId>,
    ) -> usize {
        self.propose_action(
            Action::AddProposer(proposer_address),
            opt_group_id.into_option(),
        )
    }

    /// Removes user regardless of whether it is a board member or proposer.
    #[endpoint(proposeRemoveUser)]
    fn propose_remove_user(
        &self,
        user_address: ManagedAddress,
        opt_group_id: OptionalValue<GroupId>,
    ) -> usize {
        self.propose_action(Action::RemoveUser(user_address), opt_group_id.into_option())
    }

    #[endpoint(proposeChangeQuorum)]
    fn propose_change_quorum(
        &self,
        new_quorum: usize,
        opt_group_id: OptionalValue<GroupId>,
    ) -> usize {
        self.propose_action(Action::ChangeQuorum(new_quorum), opt_group_id.into_option())
    }

    /// Propose a transaction in which the contract will perform a transfer-execute call.
    /// Can send EGLD without calling anything.
    /// Can call smart contract endpoints directly.
    /// Doesn't really work with builtin functions.
    #[endpoint(proposeTransferExecute)]
    fn propose_transfer_execute(
        &self,
        to: ManagedAddress,
        egld_amount: BigUint,
        opt_gas_limit: Option<GasLimit>,
        opt_group_id: Option<GroupId>,
        function_call: FunctionCall,
    ) -> usize {
        require!(
            egld_amount > 0 || !function_call.is_empty(),
            "proposed action has no effect"
        );

        let call_data = CallActionData {
            to,
            egld_amount,
            opt_gas_limit,
            endpoint_name: function_call.function_name,
            arguments: function_call.arg_buffer.into_vec_of_buffers(),
        };

        self.propose_action(Action::SendTransferExecute(call_data), opt_group_id)
    }

    /// Propose a transaction in which the contract will perform a transfer-execute call.
    /// Can call smart contract endpoints directly.
    /// Can use ESDTTransfer/ESDTNFTTransfer/MultiESDTTransfer to send tokens, while also optionally calling endpoints.
    /// Works well with builtin functions.
    /// Cannot simply send EGLD directly without calling anything.
    #[endpoint(proposeAsyncCall)]
    fn propose_async_call(
        &self,
        to: ManagedAddress,
        egld_amount: BigUint,
        opt_gas_limit: Option<GasLimit>,
        opt_group_id: Option<GroupId>,
        function_call: FunctionCall,
    ) -> usize {
        require!(
            egld_amount > 0 || !function_call.is_empty(),
            "proposed action has no effect"
        );

        let call_data = CallActionData {
            to,
            egld_amount,
            opt_gas_limit,
            endpoint_name: function_call.function_name,
            arguments: function_call.arg_buffer.into_vec_of_buffers(),
        };

        self.propose_action(Action::SendAsyncCall(call_data), opt_group_id)
    }

    #[endpoint(proposeSCDeployFromSource)]
    fn propose_sc_deploy_from_source(
        &self,
        amount: BigUint,
        source: ManagedAddress,
        code_metadata: CodeMetadata,
        opt_group_id: Option<GroupId>,
        arguments: MultiValueEncoded<ManagedBuffer>,
    ) -> usize {
        self.propose_action(
            Action::SCDeployFromSource {
                amount,
                source,
                code_metadata,
                arguments: arguments.into_vec_of_buffers(),
            },
            opt_group_id,
        )
    }

    #[endpoint(proposeSCUpgradeFromSource)]
    fn propose_sc_upgrade_from_source(
        &self,
        sc_address: ManagedAddress,
        amount: BigUint,
        source: ManagedAddress,
        code_metadata: CodeMetadata,
        opt_group_id: Option<GroupId>,
        arguments: MultiValueEncoded<ManagedBuffer>,
    ) -> usize {
        self.propose_action(
            Action::SCUpgradeFromSource {
                sc_address,
                amount,
                source,
                code_metadata,
                arguments: arguments.into_vec_of_buffers(),
            },
            opt_group_id,
        )
    }
}
