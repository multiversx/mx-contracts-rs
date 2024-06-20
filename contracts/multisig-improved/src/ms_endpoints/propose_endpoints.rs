use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::common_types::{
    action::{
        Action, ActionId, ActionStatus, CallActionData, DeployArgs, EsdtTransferExecuteData,
        GasLimit, GroupId,
    },
    signature::SignatureArg,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ProposeEndpointsModule:
    crate::check_signature::CheckSignatureModule
    + crate::state::StateModule
    + crate::action_types::propose::ProposeModule
{
    /// Initiates board member addition process.
    /// Can also be used to promote a proposer to board member.
    #[endpoint(proposeAddBoardMember)]
    fn propose_add_board_member(
        &self,
        board_member_address: ManagedAddress,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        self.propose_action(Action::AddBoardMember(board_member_address), opt_signature)
    }

    /// Initiates proposer addition process..
    /// Can also be used to demote a board member to proposer.
    #[endpoint(proposeAddProposer)]
    fn propose_add_proposer(
        &self,
        proposer_address: ManagedAddress,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        self.propose_action(Action::AddProposer(proposer_address), opt_signature)
    }

    /// Removes user regardless of whether it is a board member or proposer.
    #[endpoint(proposeRemoveUser)]
    fn propose_remove_user(
        &self,
        user_address: ManagedAddress,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        self.propose_action(Action::RemoveUser(user_address), opt_signature)
    }

    #[endpoint(proposeChangeQuorum)]
    fn propose_change_quorum(
        &self,
        new_quorum: usize,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        self.propose_action(Action::ChangeQuorum(new_quorum), opt_signature)
    }

    /// Propose a transaction in which the contract will perform a transfer-execute call.
    /// Can send EGLD without calling anything.
    /// Can call smart contract endpoints directly.
    /// Doesn't really work with builtin functions.
    #[allow_multiple_var_args]
    #[endpoint(proposeTransferExecute)]
    fn propose_transfer_execute(
        &self,
        to: ManagedAddress,
        egld_amount: BigUint,
        opt_gas_limit: Option<GasLimit>,
        function_call: FunctionCall,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
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

        self.propose_action(Action::SendTransferExecuteEgld(call_data), opt_signature)
    }

    #[allow_multiple_var_args]
    #[endpoint(proposeTransferExecuteEsdt)]
    fn propose_transfer_execute_esdt(
        &self,
        to: ManagedAddress,
        tokens: PaymentsVec<Self::Api>,
        opt_gas_limit: Option<GasLimit>,
        function_call: FunctionCall,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        require!(!tokens.is_empty(), "No tokens to transfer");

        let call_data = EsdtTransferExecuteData {
            to,
            tokens,
            opt_gas_limit,
            endpoint_name: function_call.function_name,
            arguments: function_call.arg_buffer.into_vec_of_buffers(),
        };

        self.propose_action(Action::SendTransferExecuteEsdt(call_data), opt_signature)
    }

    /// Propose a transaction in which the contract will perform an async call call.
    /// Can call smart contract endpoints directly.
    /// Can use ESDTTransfer/ESDTNFTTransfer/MultiESDTTransfer to send tokens, while also optionally calling endpoints.
    /// Works well with builtin functions.
    /// Cannot simply send EGLD directly without calling anything.
    #[allow_multiple_var_args]
    #[endpoint(proposeAsyncCall)]
    fn propose_async_call(
        &self,
        to: ManagedAddress,
        egld_amount: BigUint,
        opt_gas_limit: Option<GasLimit>,
        function_call: FunctionCall,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
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

        self.propose_action(Action::SendAsyncCall(call_data), opt_signature)
    }

    #[allow_multiple_var_args]
    #[endpoint(proposeSCDeployFromSource)]
    fn propose_sc_deploy_from_source(
        &self,
        amount: BigUint,
        source: ManagedAddress,
        code_metadata: CodeMetadata,
        opt_signature: Option<SignatureArg<Self::Api>>,
        arguments: MultiValueEncoded<ManagedBuffer>,
    ) -> ActionId {
        self.propose_action(
            Action::SCDeployFromSource(DeployArgs {
                amount,
                source,
                code_metadata,
                arguments: arguments.into_vec_of_buffers(),
            }),
            opt_signature.into(),
        )
    }

    #[allow_multiple_var_args]
    #[endpoint(proposeSCUpgradeFromSource)]
    fn propose_sc_upgrade_from_source(
        &self,
        sc_address: ManagedAddress,
        amount: BigUint,
        source: ManagedAddress,
        code_metadata: CodeMetadata,
        opt_signature: Option<SignatureArg<Self::Api>>,
        arguments: MultiValueEncoded<ManagedBuffer>,
    ) -> ActionId {
        self.propose_action(
            Action::SCUpgradeFromSource {
                sc_address,
                args: DeployArgs {
                    amount,
                    source,
                    code_metadata,
                    arguments: arguments.into_vec_of_buffers(),
                },
            },
            opt_signature.into(),
        )
    }

    #[endpoint(proposeBatch)]
    fn propose_batch(&self, actions: MultiValueEncoded<Action<Self::Api>>) -> GroupId {
        let group_id = self.last_action_group_id().get() + 1;
        require!(!actions.is_empty(), "No actions");

        let (caller_id, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_propose::<Self::Api>();

        let mut action_mapper = self.action_mapper();
        let mut action_groups_mapper = self.action_groups(group_id);
        self.action_group_status(group_id)
            .set(ActionStatus::Available);

        require!(
            action_groups_mapper.is_empty(),
            "cannot add actions to an already existing batch"
        );

        for action in actions {
            self.require_valid_action_type(&action);
            self.ensure_valid_transfer_action(&action);

            let action_id = action_mapper.push(&action);
            if caller_role.can_sign() {
                let _ = self.action_signer_ids(action_id).insert(caller_id);
            }

            let _ = action_groups_mapper.insert(action_id);
            self.group_for_action(action_id).set(group_id);
        }

        self.last_action_group_id().set(group_id);

        group_id
    }
}
