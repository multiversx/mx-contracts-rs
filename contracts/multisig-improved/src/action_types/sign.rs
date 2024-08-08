use crate::common_types::action::ActionId;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SignModule:
    crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
    + super::external_module::ExternalModuleModule
    + super::propose::ProposeModule
    + super::perform::PerformModule
    + super::execute_action::ExecuteActionModule
    + crate::ms_endpoints::callbacks::CallbacksModule
    + crate::external::events::EventsModule
    + crate::check_signature::CheckSignatureModule
{
    fn unsign_action(&self, action_id: ActionId, caller_id: AddressId) {
        self.require_action_exists(action_id);

        let _ = self.action_signer_ids(action_id).swap_remove(&caller_id);
    }

    fn add_signatures(&self, action_id: ActionId, board_members: &ManagedVec<AddressId>) {
        let mut mapper = self.action_signer_ids(action_id);
        for board_member in board_members {
            let _ = mapper.insert(board_member);
        }
    }
}
