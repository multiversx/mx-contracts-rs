use crate::common_types::action::{ActionId, ActionStatus, GroupId, Nonce};
use crate::common_types::{action::Action, user_role::UserRole};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StateModule {
    /// Minimum number of signatures needed to perform any action.
    #[view(getQuorum)]
    #[storage_mapper("quorum_ids")]
    fn quorum(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("user_ids")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("userNonce")]
    fn user_nonce(&self, user_id: AddressId) -> SingleValueMapper<Nonce>;

    #[storage_mapper("quorum_for_action")]
    fn quorum_for_action(&self, action_id: ActionId) -> SingleValueMapper<usize>;

    #[storage_mapper("user_role")]
    fn user_id_to_role(&self, user_id: AddressId) -> SingleValueMapper<UserRole>;

    /// Denormalized board member count.
    /// It is kept in sync with the user list by the contract.
    #[view(getNumBoardMembers)]
    #[storage_mapper("num_board_members")]
    fn num_board_members(&self) -> SingleValueMapper<usize>;

    #[view(getNumGroups)]
    #[storage_mapper("num_groups")]
    fn num_groups(&self) -> SingleValueMapper<usize>;

    /// Denormalized proposer count.
    /// It is kept in sync with the user list by the contract.
    #[view(getNumProposers)]
    #[storage_mapper("num_proposers")]
    fn num_proposers(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("action_data")]
    fn action_mapper(&self) -> VecMapper<Action<Self::Api>>;

    #[view(getActionGroup)]
    #[storage_mapper("action_groups")]
    fn action_groups(&self, group_id: GroupId) -> UnorderedSetMapper<ActionId>;

    #[view(getLastGroupActionId)]
    #[storage_mapper("last_action_group_id")]
    fn last_action_group_id(&self) -> SingleValueMapper<GroupId>;

    #[view(getActionGroup)]
    #[storage_mapper("action_group_status")]
    fn action_group_status(&self, group_id: GroupId) -> SingleValueMapper<ActionStatus>;

    #[storage_mapper("group_for_action")]
    fn group_for_action(&self, action_id: ActionId) -> SingleValueMapper<GroupId>;

    #[storage_mapper("action_signer_ids")]
    fn action_signer_ids(&self, action_id: ActionId) -> UnorderedSetMapper<AddressId>;
}
