use multiversx_sc::api::SHA256_RESULT_LEN;

use crate::common_types::{
    action::{Action, ActionId, GroupId, Nonce},
    signature::{ActionType, DecomposeResultType, ItemToSign, Signature, SignatureMultiArg},
};

multiversx_sc::imports!();

static ENCODING_NONCE_ERR_MSG: &[u8] = b"Error encoding user nonce to buffer";

#[multiversx_sc::module]
pub trait CheckSignatureModule: crate::state::StateModule {
    fn check_single_action_signatures(
        &self,
        action_id: ActionId,
        signatures: MultiValueEncoded<SignatureMultiArg<Self::Api>>,
    ) -> ManagedVec<AddressId> {
        let mut board_members = ManagedVec::new();

        let action = self.action_mapper().get_unchecked(action_id);
        for sig_multi_arg in signatures {
            let decompose_result =
                self.decompose_and_check_action_type(sig_multi_arg, ActionType::SimpleAction);

            let user_id = decompose_result.user_id;
            self.check_signature_by_item_to_sign(decompose_result, ItemToSign::Action(&action));

            board_members.push(user_id);
        }

        board_members
    }

    fn check_group_signatures(
        &self,
        group_id: GroupId,
        signatures: MultiValueEncoded<SignatureMultiArg<Self::Api>>,
    ) -> ManagedVec<AddressId> {
        let mut board_members = ManagedVec::new();

        for sig_multi_arg in signatures {
            let decompose_result =
                self.decompose_and_check_action_type(sig_multi_arg, ActionType::Group);

            let user_id = decompose_result.user_id;
            self.check_signature_by_item_to_sign(decompose_result, ItemToSign::Group(group_id));

            board_members.push(user_id);
        }

        board_members
    }

    fn decompose_and_check_action_type(
        &self,
        sig_multi_arg: SignatureMultiArg<Self::Api>,
        requested_action_type: ActionType,
    ) -> DecomposeResultType<Self::Api> {
        let (board_member, nonce, action_type, signature_type, raw_sig_bytes) =
            sig_multi_arg.into_tuple();
        let (user_id, user_role) = self.get_id_and_role(&board_member);
        user_role.require_can_sign::<Self::Api>();

        let next_user_nonce = self.get_and_increment_user_nonce(&board_member);
        require!(nonce == next_user_nonce, "Invalid nonce");

        action_type.require_is_type::<Self::Api>(requested_action_type);

        DecomposeResultType {
            board_member,
            user_id,
            nonce,
            signature_type,
            raw_sig_bytes,
        }
    }

    fn check_signature_by_item_to_sign(
        &self,
        decompose_result: DecomposeResultType<Self::Api>,
        item_to_sign: ItemToSign<Self::Api>,
    ) {
        let bytes_to_sign = match item_to_sign {
            ItemToSign::Action(action) => self.serialize_and_hash_action(
                action,
                &decompose_result.board_member,
                decompose_result.nonce,
            ),
            ItemToSign::Group(group_id) => self.serialize_and_hash_group(
                group_id,
                &decompose_result.board_member,
                decompose_result.nonce,
            ),
        };
        let signature_struct = Signature {
            signature_type: decompose_result.signature_type,
            raw_sig_bytes: decompose_result.raw_sig_bytes,
        };
        signature_struct.check_signature_by_type(
            &decompose_result.board_member,
            bytes_to_sign.as_managed_buffer(),
        );
    }

    fn serialize_and_hash_action(
        &self,
        action: &Action<Self::Api>,
        signer: &ManagedAddress,
        user_nonce: Nonce,
    ) -> ManagedByteArray<SHA256_RESULT_LEN> {
        let mut all_data = signer.as_managed_buffer().clone();
        let nonce_encode_result = user_nonce.dep_encode(&mut all_data);
        require!(nonce_encode_result.is_ok(), ENCODING_NONCE_ERR_MSG);

        let action_encode_result = action.dep_encode(&mut all_data);
        require!(
            action_encode_result.is_ok(),
            "Error encoding action to buffer"
        );

        self.crypto().sha256(all_data)
    }

    fn serialize_and_hash_group(
        &self,
        group_id: GroupId,
        signer: &ManagedAddress,
        user_nonce: Nonce,
    ) -> ManagedByteArray<SHA256_RESULT_LEN> {
        let mut all_data = signer.as_managed_buffer().clone();
        let nonce_encode_result = user_nonce.dep_encode(&mut all_data);
        require!(nonce_encode_result.is_ok(), ENCODING_NONCE_ERR_MSG);

        let group_encode_result = group_id.dep_encode(&mut all_data);
        require!(
            group_encode_result.is_ok(),
            "Error encoding Group ID to buffer"
        );

        self.crypto().sha256(all_data)
    }
}
