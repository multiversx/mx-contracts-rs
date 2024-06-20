use crate::common_types::action::ActionId;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DiscardEndpointsModule:
    crate::state::StateModule
    + crate::action_types::propose::ProposeModule
    + crate::action_types::sign::SignModule
    + crate::action_types::perform::PerformModule
    + crate::action_types::execute_action::ExecuteActionModule
    + crate::action_types::discard::DiscardActionModule
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
}
