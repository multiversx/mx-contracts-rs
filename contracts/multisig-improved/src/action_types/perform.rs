use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::common_types::action::{
    ActionFullInfo, ActionId, ActionStatus, CallActionData, EsdtTransferExecuteData,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PerformModule:
    crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
    + super::external_module::ExternalModuleModule
    + crate::external::events::EventsModule
    + super::execute_action::ExecuteActionModule
    + crate::ms_endpoints::callbacks::CallbacksModule
{
    fn perform_action_by_id(&self, action_id: ActionId) -> OptionalValue<ManagedAddress> {
        let action = self.action_mapper().get(action_id);

        let group_id = self.group_for_action(action_id).get();
        if group_id != 0 {
            let group_status = self.action_group_status(group_id).get();
            require!(
                group_status == ActionStatus::Available,
                "cannot perform actions of an aborted batch"
            );
        }

        self.start_perform_action_event(&ActionFullInfo {
            action_id,
            action_data: action.clone(),
            signers: self.get_action_signers(action_id),
            group_id,
        });

        // clean up storage
        // happens before actual execution, because the match provides the return on each branch
        // syntax aside, the async_call_raw kills contract execution so cleanup cannot happen afterwards
        self.clear_action(action_id);

        let opt_address = self.try_execute_deploy(action_id, &action);
        if opt_address.is_some() {
            return opt_address;
        }

        self.execute_action_by_type(action_id, action);

        OptionalValue::None
    }

    fn try_perform_egld_action_directly(
        &self,
        action_id: ActionId,
        call_data: &CallActionData<Self::Api>,
    ) -> bool {
        let can_execute = self.can_execute_action(
            call_data.to.clone(),
            call_data.egld_amount.clone(),
            PaymentsVec::new(),
        );
        if !can_execute {
            return false;
        }

        let _ = self.perform_action_by_id(action_id);

        true
    }

    fn try_perform_esdt_action_directly(
        &self,
        action_id: ActionId,
        call_data: &EsdtTransferExecuteData<Self::Api>,
    ) -> bool {
        let can_execute = self.can_execute_action(
            call_data.to.clone(),
            BigUint::zero(),
            call_data.tokens.clone(),
        );
        if !can_execute {
            return false;
        }

        let _ = self.perform_action_by_id(action_id);

        true
    }

    fn try_perform_action(&self, action_id: ActionId) -> OptionalValue<ManagedAddress> {
        let (_, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_perform_action::<Self::Api>();

        if !self.quorum_reached(action_id) {
            return OptionalValue::None;
        }

        let group_id = self.group_for_action(action_id).get();
        require!(group_id == 0, "May not execute this action by itself");

        self.perform_action_by_id(action_id)
    }

    fn require_same_shard(&self, sc_address: &ManagedAddress) {
        let own_address = self.blockchain().get_sc_address();
        let own_shard = self.blockchain().get_shard_of_address(&own_address);
        let sc_shard = self.blockchain().get_shard_of_address(sc_address);
        require!(own_shard == sc_shard, "Must be same shard");
    }
}
