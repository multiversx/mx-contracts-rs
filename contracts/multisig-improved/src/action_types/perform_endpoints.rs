use crate::common_types::action::{ActionId, ActionStatus, GroupId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PerformEndpointsModule:
    crate::state::StateModule
    + crate::external::events::EventsModule
    + super::perform::PerformModule
    + super::execute_action::ExecuteActionModule
    + super::callbacks::CallbacksModule
{
    /// Proposers and board members use this to launch signed actions.
    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: ActionId) -> OptionalValue<ManagedAddress> {
        let (_, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_perform_action::<Self::Api>();

        require!(
            self.quorum_reached(action_id),
            "quorum has not been reached"
        );

        let group_id = self.group_for_action(action_id).get();
        require!(group_id == 0, "May not execute this action by itself");

        self.perform_action(action_id)
    }

    /// Perform all the actions in the given batch
    #[endpoint(performBatch)]
    fn perform_batch(&self, group_id: GroupId) {
        let (_, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_perform_action::<Self::Api>();

        let group_status = self.action_group_status(group_id).get();
        require!(
            group_status == ActionStatus::Available,
            "cannot perform actions of an aborted batch"
        );

        let mapper = self.action_groups(group_id);
        require!(!mapper.is_empty(), "Invalid group ID");

        let mut action_ids = ManagedVec::<Self::Api, _>::new();
        for action_id in mapper.iter() {
            action_ids.push(action_id);
        }

        for action_id in &action_ids {
            require!(
                self.quorum_reached(action_id),
                "quorum has not been reached"
            );

            let _ = self.perform_action(action_id);
        }
    }
}
