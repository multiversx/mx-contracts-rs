use crate::common_types::{
    action::{ActionId, ActionStatus, GroupId},
    signature::SignatureArg,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SignEndpointsModule:
    crate::common_functions::CommonFunctionsModule
    + crate::state::StateModule
    + crate::action_types::external_module::ExternalModuleModule
    + crate::action_types::propose::ProposeModule
    + crate::action_types::perform::PerformModule
    + crate::action_types::execute_action::ExecuteActionModule
    + crate::action_types::sign::SignModule
    + super::callbacks::CallbacksModule
    + crate::external::events::EventsModule
    + crate::check_signature::CheckSignatureModule
{
    /// Used by board members to sign actions.
    #[endpoint]
    fn sign(&self, action_id: ActionId, signatures: MultiValueEncoded<SignatureArg<Self::Api>>) {
        self.require_action_exists(action_id);

        let group_id = self.group_for_action(action_id).get();
        if group_id != 0 {
            let group_status = self.action_group_status(group_id).get();
            require!(
                group_status == ActionStatus::Available,
                "cannot sign actions of an aborted batch"
            );
        }

        let user_ids = self.check_single_action_signatures(action_id, signatures);
        self.add_signatures(action_id, &user_ids);
    }

    /// Sign all the actions in the given batch
    /// Signatures must be given in order of the action IDs inside batch, even if it was already signed
    #[endpoint(signBatch)]
    fn sign_batch(
        &self,
        group_id: GroupId,
        signatures: MultiValueEncoded<SignatureArg<Self::Api>>,
    ) {
        let group_status = self.action_group_status(group_id).get();
        require!(
            group_status == ActionStatus::Available,
            "cannot sign actions of an aborted batch"
        );

        let mapper = self.action_groups(group_id);
        require!(!mapper.is_empty(), "Invalid group ID");

        let user_ids = self.check_group_signatures(group_id, signatures);
        for action_id in mapper.iter() {
            self.require_action_exists(action_id);

            self.add_signatures(action_id, &user_ids);
        }
    }

    #[endpoint(signAndPerform)]
    fn sign_and_perform(
        &self,
        action_id: ActionId,
        signatures: MultiValueEncoded<SignatureArg<Self::Api>>,
    ) -> OptionalValue<ManagedAddress> {
        self.sign(action_id, signatures);
        self.try_perform_action(action_id)
    }

    #[endpoint(signBatchAndPerform)]
    fn sign_batch_and_perform(
        &self,
        group_id: GroupId,
        signatures: MultiValueEncoded<SignatureArg<Self::Api>>,
    ) {
        self.sign_batch(group_id, signatures);

        let (_, caller_role) = self.get_caller_id_and_role();
        require!(
            caller_role.can_perform_action(),
            "only board members and proposers can perform actions"
        );

        // Copy action_ids before executing them since perform_action does a swap_remove
        // clearing the last item
        let mut action_ids = ManagedVec::<Self::Api, _>::new();
        for action_id in self.action_groups(group_id).iter() {
            require!(
                self.quorum_reached(action_id),
                "Quorum not reached for action"
            );

            action_ids.push(action_id);
        }

        for action_id in &action_ids {
            let _ = self.perform_action_by_id(action_id);
        }
    }

    /// Board members can withdraw their signatures if they no longer desire for the action to be executed.
    /// Actions that are left with no valid signatures can be then deleted to free up storage.
    #[endpoint]
    fn unsign(&self, action_id: ActionId) {
        let (caller_id, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_unsign::<Self::Api>();

        self.unsign_action(action_id, caller_id);
    }

    /// Unsign all actions with the given IDs
    #[endpoint(unsignBatch)]
    fn unsign_batch(&self, group_id: GroupId) {
        let (caller_id, caller_role) = self.get_caller_id_and_role();
        caller_role.require_can_unsign::<Self::Api>();

        let mapper = self.action_groups(group_id);
        require!(!mapper.is_empty(), "Invalid group ID");

        for action_id in mapper.iter() {
            self.unsign_action(action_id, caller_id);
        }
    }

    #[endpoint(unsignForOutdatedBoardMembers)]
    fn unsign_for_outdated_board_members(
        &self,
        action_id: ActionId,
        outdated_board_members: MultiValueEncoded<AddressId>,
    ) {
        let mut board_members_to_remove = ManagedVec::<Self::Api, u64>::new();
        if outdated_board_members.is_empty() {
            for signer_id in self.action_signer_ids(action_id).iter() {
                if !self.user_id_to_role(signer_id).get().can_sign() {
                    board_members_to_remove.push(signer_id);
                }
            }
        } else {
            for signer_id in outdated_board_members.into_iter() {
                if !self.user_id_to_role(signer_id).get().can_sign() {
                    board_members_to_remove.push(signer_id);
                }
            }
        }

        for member in board_members_to_remove.iter() {
            self.action_signer_ids(action_id).swap_remove(&member);
        }
    }
}
