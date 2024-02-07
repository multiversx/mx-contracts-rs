use crate::multisig_state::ActionId;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MultisigSignModule:
    crate::multisig_state::MultisigStateModule
    + crate::multisig_propose::MultisigProposeModule
    + crate::multisig_perform::MultisigPerformModule
    + crate::multisig_events::MultisigEventsModule
{
    /// Used by board members to sign actions.
    #[endpoint]
    fn sign(&self, action_id: ActionId) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let (caller_id, caller_role) = self.get_caller_id_and_role();
        require!(caller_role.can_sign(), "only board members can sign");

        let _ = self.action_signer_ids(action_id).insert(caller_id);
    }

    /// Sign all the actions with the given IDs
    #[endpoint(signBatch)]
    fn sign_batch(&self, action_ids: MultiValueEncoded<ActionId>) {
        let (caller_id, caller_role) = self.get_caller_id_and_role();
        require!(caller_role.can_sign(), "only board members can sign");

        for action_id in action_ids {
            require!(
                !self.action_mapper().item_is_empty_unchecked(action_id),
                "action does not exist"
            );

            let _ = self.action_signer_ids(action_id).insert(caller_id);
        }
    }

    #[endpoint(signAndPerform)]
    fn sign_and_perform(&self, action_id: ActionId) -> OptionalValue<ManagedAddress> {
        self.sign(action_id);
        self.perform_action_endpoint(action_id)
    }

    #[endpoint(signBatchAndPerform)]
    fn sign_batch_and_perform(&self, action_ids: MultiValueEncoded<ActionId>) {
        self.sign_batch(action_ids.clone());

        for action_id in action_ids {
            let _ = self.perform_action_endpoint(action_id);
        }
    }

    /// Board members can withdraw their signatures if they no longer desire for the action to be executed.
    /// Actions that are left with no valid signatures can be then deleted to free up storage.
    #[endpoint]
    fn unsign(&self, action_id: ActionId) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let (caller_id, caller_role) = self.get_caller_id_and_role();
        require!(caller_role.can_sign(), "only board members can un-sign");

        let _ = self.action_signer_ids(action_id).swap_remove(&caller_id);
    }

    /// Returns `true` (`1`) if the user has signed the action.
    /// Does not check whether or not the user is still a board member and the signature valid.
    #[view]
    fn signed(&self, user: ManagedAddress, action_id: ActionId) -> bool {
        let user_id = self.user_mapper().get_user_id(&user);
        if user_id == 0 {
            false
        } else {
            self.action_signer_ids(action_id).contains(&user_id)
        }
    }
}
