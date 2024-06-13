use crate::common_types::action::ActionId;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, Copy, PartialEq, Eq, Debug)]
pub enum UserRole {
    None,
    Proposer,
    BoardMember,
}

impl UserRole {
    pub fn can_propose(&self) -> bool {
        matches!(*self, UserRole::BoardMember | UserRole::Proposer)
    }

    pub fn can_perform_action(&self) -> bool {
        self.can_propose()
    }

    pub fn can_discard_action(&self) -> bool {
        self.can_propose()
    }

    pub fn can_sign(&self) -> bool {
        matches!(*self, UserRole::BoardMember)
    }

    pub fn has_no_role(&self) -> bool {
        matches!(*self, UserRole::None)
    }

    pub fn require_can_propose<M: ManagedTypeApi>(&self) {
        if !self.can_propose() {
            M::error_api_impl().signal_error(b"only board members and proposers can propose");
        }
    }

    pub fn require_can_sign<M: ManagedTypeApi>(&self) {
        if !self.can_sign() {
            M::error_api_impl().signal_error(b"only board members can sign");
        }
    }

    pub fn require_can_unsign<M: ManagedTypeApi>(&self) {
        if !self.can_sign() {
            M::error_api_impl().signal_error(b"only board members can un-sign");
        }
    }

    pub fn require_can_perform_action<M: ManagedTypeApi>(&self) {
        if !self.can_perform_action() {
            M::error_api_impl()
                .signal_error(b"only board members and proposers can perform actions");
        }
    }

    pub fn require_can_discard_action<M: ManagedTypeApi>(&self) {
        if !self.can_discard_action() {
            M::error_api_impl()
                .signal_error(b"only board members and proposers can discard actions");
        }
    }
}

fn usize_add_isize(value: &mut usize, delta: isize) {
    *value = (*value as isize + delta) as usize;
}

/// Can be used to:
/// - create new user (board member / proposer)
/// - remove user (board member / proposer)
/// - reactivate removed user
/// - convert between board member and proposer
/// Will keep the board size and proposer count in sync.
pub fn change_user_role<Sc: crate::state::StateModule + crate::external::events::EventsModule>(
    sc_ref: &Sc,
    action_id: ActionId,
    user_address: ManagedAddress<Sc::Api>,
    new_role: UserRole,
) {
    let user_id = if new_role == UserRole::None {
        // avoid creating a new user just to delete it
        let user_id = sc_ref.user_ids().get_id(&user_address);
        if user_id == 0 {
            return;
        }

        user_id
    } else {
        sc_ref.user_ids().get_id_or_insert(&user_address)
    };

    let user_id_to_role_mapper = sc_ref.user_id_to_role(user_id);
    let old_role = user_id_to_role_mapper.get();
    user_id_to_role_mapper.set(new_role);

    sc_ref.perform_change_user_event(action_id, &user_address, old_role, new_role);

    // update board size
    let mut board_members_delta = 0isize;
    if old_role == UserRole::BoardMember {
        board_members_delta -= 1;
    }
    if new_role == UserRole::BoardMember {
        board_members_delta += 1;
    }
    if board_members_delta != 0 {
        sc_ref
            .num_board_members()
            .update(|value| usize_add_isize(value, board_members_delta));
    }

    let mut proposers_delta = 0isize;
    if old_role == UserRole::Proposer {
        proposers_delta -= 1;
    }
    if new_role == UserRole::Proposer {
        proposers_delta += 1;
    }
    if proposers_delta != 0 {
        sc_ref
            .num_proposers()
            .update(|value| usize_add_isize(value, proposers_delta));
    }
}
