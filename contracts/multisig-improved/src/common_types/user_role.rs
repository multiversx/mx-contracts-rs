use multiversx_sc::api::{ErrorApiImpl, ManagedTypeApi};

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
