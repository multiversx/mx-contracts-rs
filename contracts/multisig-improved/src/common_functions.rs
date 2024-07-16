use crate::common_types::{
    action::{ActionId, Nonce},
    user_role::UserRole,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CommonFunctionsModule: crate::state::StateModule {
    /// Returns `true` (`1`) if `getActionValidSignerCount >= getQuorum`.
    #[view(quorumReached)]
    fn quorum_reached(&self, action_id: ActionId) -> bool {
        let quorum = self.quorum_for_action(action_id).get();
        let valid_signers_count = self.get_action_valid_signer_count(action_id);
        valid_signers_count >= quorum
    }

    fn get_action_valid_signer_count(&self, action_id: ActionId) -> usize {
        let signer_ids = self.action_signer_ids(action_id);
        signer_ids
            .iter()
            .filter(|signer_id| {
                let signer_role = self.user_id_to_role(*signer_id).get();
                signer_role.can_sign()
            })
            .count()
    }

    fn get_action_signers(&self, action_id: ActionId) -> ManagedVec<ManagedAddress> {
        let signer_ids = self.action_signer_ids(action_id);
        let mut signers = ManagedVec::new();
        for signer_id in signer_ids.iter() {
            let opt_user_address = self.user_ids().get_address(signer_id);
            let address = unsafe { opt_user_address.unwrap_unchecked() };
            signers.push(address);
        }

        signers
    }

    fn get_caller_id_and_role(&self) -> (AddressId, UserRole) {
        let caller_address = self.blockchain().get_caller();
        self.get_id_and_role(&caller_address)
    }

    fn get_id_and_role(&self, user_address: &ManagedAddress) -> (AddressId, UserRole) {
        let user_id = self.user_ids().get_id(user_address);
        let user_role = self.user_id_to_role(user_id).get();

        (user_id, user_role)
    }

    fn get_and_increment_user_nonce(&self, user_address: &ManagedAddress) -> Nonce {
        let user_id = self.user_ids().get_id_non_zero(user_address);

        let mut output_nonce = 0;
        self.user_nonce(user_id).update(|user_nonce| {
            output_nonce = *user_nonce;
            *user_nonce += 1;
        });

        output_nonce
    }

    fn require_action_exists(&self, action_id: ActionId) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );
    }
}
