use crate::common_types::{action::ActionFullInfo, user_role::UserRole};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ViewsModule:
    crate::state::StateModule
    + crate::action_types::propose::ProposeModule
    + crate::action_types::sign::SignModule
    + crate::action_types::perform::PerformModule
    + super::events::EventsModule
{
    /// Iterates through all actions and retrieves those that are still pending.
    /// Serialized full action data:
    /// - the action id
    /// - the serialized action data
    /// - (number of signers followed by) list of signer addresses.
    #[label("multisig-external-view")]
    #[allow_multiple_var_args]
    #[view(getPendingActionFullInfo)]
    fn get_pending_action_full_info(
        &self,
        opt_range: OptionalValue<(usize, usize)>,
    ) -> MultiValueEncoded<ActionFullInfo<Self::Api>> {
        let mut result = MultiValueEncoded::new();
        let action_last_index = self.get_action_last_index();
        let action_mapper = self.action_mapper();
        let mut index_of_first_action = 1;
        let mut index_of_last_action = action_last_index;
        if let OptionalValue::Some((count, first_action_id)) = opt_range {
            require!(
                first_action_id <= action_last_index,
                "first_action_id needs to be within the range of the available action ids"
            );
            index_of_first_action = first_action_id;

            require!(
                index_of_first_action + count <= action_last_index,
                "cannot exceed the total number of actions"
            );
            index_of_last_action = index_of_first_action + count;
        }
        for action_id in index_of_first_action..=index_of_last_action {
            let action_data = action_mapper.get(action_id);
            if action_data.is_pending() {
                result.push(ActionFullInfo {
                    action_id,
                    action_data,
                    signers: self.get_action_signers(action_id),
                    group_id: self.group_for_action(action_id).get(),
                });
            }
        }
        result
    }

    /// Indicates user rights.
    /// `0` = no rights,
    /// `1` = can propose, but not sign,
    /// `2` = can propose and sign.
    #[label("multisig-external-view")]
    #[view(userRole)]
    fn user_role(&self, user: ManagedAddress) -> UserRole {
        let user_id = self.user_mapper().get_user_id(&user);
        if user_id == 0 {
            return UserRole::None;
        }

        self.user_id_to_role(user_id).get()
    }

    /// Lists all users that can sign actions.
    #[label("multisig-external-view")]
    #[view(getAllBoardMembers)]
    fn get_all_board_members(&self) -> MultiValueEncoded<ManagedAddress> {
        self.get_all_users_with_role(UserRole::BoardMember)
    }

    /// Lists all proposers that are not board members.
    #[label("multisig-external-view")]
    #[view(getAllProposers)]
    fn get_all_proposers(&self) -> MultiValueEncoded<ManagedAddress> {
        self.get_all_users_with_role(UserRole::Proposer)
    }

    fn get_all_users_with_role(&self, role: UserRole) -> MultiValueEncoded<ManagedAddress> {
        let mut result = MultiValueEncoded::new();
        let num_users = self.user_mapper().get_user_count();
        for user_id in 1..=num_users {
            if self.user_id_to_role(user_id).get() == role {
                if let Some(address) = self.user_mapper().get_user_address(user_id) {
                    result.push(address);
                }
            }
        }

        result
    }
}
