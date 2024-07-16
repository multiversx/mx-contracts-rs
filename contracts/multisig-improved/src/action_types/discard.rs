use crate::common_types::action::{ActionId, ActionStatus};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DiscardActionModule:
    crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
    + super::external_module::ExternalModuleModule
    + super::propose::ProposeModule
    + super::sign::SignModule
    + super::perform::PerformModule
    + super::execute_action::ExecuteActionModule
    + crate::ms_endpoints::callbacks::CallbacksModule
    + crate::check_signature::CheckSignatureModule
    + crate::external::events::EventsModule
{
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
