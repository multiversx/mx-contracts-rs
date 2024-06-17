use crate::common_types::{
    action::{Action, ActionId},
    signature::SignatureArg,
};

multiversx_sc::imports!();

static ALL_TRANSFER_EXEC_SAME_SHARD_ERR_MSG: &[u8] = b"All transfer exec must be to the same shard";

#[multiversx_sc::module]
pub trait ProposeModule:
    crate::check_signature::CheckSignatureModule
    + crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
{
    fn propose_action(
        &self,
        action: Action<Self::Api>,
        opt_signature: OptionalValue<SignatureArg<Self::Api>>,
    ) -> ActionId {
        let caller = self.blockchain().get_caller();
        let proposer = match opt_signature {
            OptionalValue::Some(sig_arg) => {
                let proposer = sig_arg.user_address.clone();
                self.check_proposal_signature(&action, sig_arg);

                proposer
            }
            OptionalValue::None => caller,
        };

        let (proposer_id, proposer_role) = self.get_id_and_role(&proposer);
        proposer_role.require_can_propose::<Self::Api>();

        let action_id = self.action_mapper().push(&action);
        let quorum = self.quorum().get();
        self.quorum_for_action(action_id).set(quorum);

        if proposer_role.can_sign() {
            // also sign
            // since the action is newly created, the proposer can be the only signer
            let _ = self.action_signer_ids(action_id).insert(proposer_id);
        }

        action_id
    }

    fn require_valid_action_type(&self, action: &Action<Self::Api>) {
        require!(
            !action.is_nothing() && !action.is_async_call() && !action.is_sc_upgrade(),
            "Invalid action"
        );
    }

    fn ensure_valid_transfer_action(&self, action: &Action<Self::Api>) {
        let own_sc_address = self.blockchain().get_sc_address();
        let own_shard = self.blockchain().get_shard_of_address(&own_sc_address);
        match action {
            Action::SendTransferExecuteEgld(call_data) => {
                let other_sc_shard = self.blockchain().get_shard_of_address(&call_data.to);
                require!(
                    call_data.egld_amount > 0 || !call_data.endpoint_name.is_empty(),
                    "proposed action has no effect"
                );
                require!(
                    own_shard == other_sc_shard,
                    ALL_TRANSFER_EXEC_SAME_SHARD_ERR_MSG
                );
            }
            Action::SendTransferExecuteEsdt(call_data) => {
                require!(!call_data.tokens.is_empty(), "No tokens to transfer");

                let other_sc_shard = self.blockchain().get_shard_of_address(&call_data.to);
                require!(
                    own_shard == other_sc_shard,
                    ALL_TRANSFER_EXEC_SAME_SHARD_ERR_MSG
                );
            }
            _ => {}
        }
    }
}
