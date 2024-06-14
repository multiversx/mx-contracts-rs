use crate::common_types::action::{ActionId, ActionStatus};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DiscardActionModule:
    crate::state::StateModule
    + super::propose::ProposeModule
    + super::sign::SignModule
    + super::perform::PerformModule
    + super::execute_action::ExecuteActionModule
    + super::callbacks::CallbacksModule
    + crate::check_signature::CheckSignatureModule
    + crate::external::events::EventsModule
{
    /// Clears storage pertaining to an action that is no longer supposed to be executed.
    /// Any signatures that the action received must first be removed, via `unsign`.
    /// Otherwise this endpoint would be prone to abuse.
    #[endpoint(discardAction)]
    fn discard_action_endpoint(&self, action_id: ActionId) {
        let (_, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_discard_action::<Self::Api>();

        self.discard_action(action_id);
    }

    /// Discard all the actions with the given IDs
    #[endpoint(discardBatch)]
    fn discard_batch(&self, action_ids: MultiValueEncoded<ActionId>) {
        let (_, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_discard_action::<Self::Api>();

        for action_id in action_ids {
            self.discard_action(action_id);
        }
    }

    fn discard_action(&self, action_id: ActionId) {
        require!(
            self.get_action_valid_signer_count(action_id) == 0,
            "cannot discard action with valid signatures"
        );
        self.abort_batch_of_action(action_id);
        self.clear_action(action_id);
    }

    fn abort_batch_of_action(&self, action_id: ActionId) {
        let batch_id = self.group_for_action(action_id).get();
        if batch_id != 0 {
            self.action_group_status(batch_id)
                .set(ActionStatus::Aborted);
        }
    }
}
