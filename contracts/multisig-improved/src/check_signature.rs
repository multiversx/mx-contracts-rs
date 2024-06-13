use multiversx_sc::api::SHA256_RESULT_LEN;

use crate::common_types::{
    action::{Action, ActionId, GroupId, Nonce},
    signature::{ActionType, ItemToSign, Signature, SignatureArg},
};

multiversx_sc::imports!();

static ENCODING_NONCE_ERR_MSG: &[u8] = b"Error encoding user nonce to buffer";
static ENCODING_ACTION_TYPE_ERR_MSG: &[u8] = b"Error encoding action type to buffer";

#[multiversx_sc::module]
pub trait CheckSignatureModule: crate::state::StateModule {
    fn check_single_action_signatures(
        &self,
        action_id: ActionId,
        signatures: MultiValueEncoded<SignatureArg<Self::Api>>,
    ) -> ManagedVec<AddressId> {
        let mut board_members = ManagedVec::new();

        let id_mapper = self.user_ids();
        let action = self.action_mapper().get_unchecked(action_id);
        for sig_arg in signatures {
            let user_id = id_mapper.get_id_non_zero(&sig_arg.board_member);

            self.check_base_signature_validity(&sig_arg, ActionType::SimpleAction);
            self.check_signature_by_item_to_sign(sig_arg, ItemToSign::Action(&action));

            board_members.push(user_id);
        }

        board_members
    }

    fn check_group_signatures(
        &self,
        group_id: GroupId,
        signatures: MultiValueEncoded<SignatureArg<Self::Api>>,
    ) -> ManagedVec<AddressId> {
        let mut board_members = ManagedVec::new();

        let id_mapper = self.user_ids();
        for sig_arg in signatures {
            let user_id = id_mapper.get_id_non_zero(&sig_arg.board_member);

            self.check_base_signature_validity(&sig_arg, ActionType::Group);
            self.check_signature_by_item_to_sign(sig_arg, ItemToSign::Group(group_id));

            board_members.push(user_id);
        }

        board_members
    }

    fn check_base_signature_validity(
        &self,
        sig_arg: &SignatureArg<Self::Api>,
        requested_action_type: ActionType,
    ) {
        let (_, user_role) = self.get_id_and_role(&sig_arg.board_member);
        user_role.require_can_sign::<Self::Api>();

        let next_user_nonce = self.get_and_increment_user_nonce(&sig_arg.board_member);
        require!(sig_arg.nonce == next_user_nonce, "Invalid nonce");

        sig_arg
            .action_type
            .require_is_type::<Self::Api>(requested_action_type);
    }

    fn check_signature_by_item_to_sign(
        &self,
        sig_arg: SignatureArg<Self::Api>,
        item_to_sign: ItemToSign<Self::Api>,
    ) {
        let bytes_to_sign = match item_to_sign {
            ItemToSign::Action(action) => {
                self.serialize_and_hash_action(action, &sig_arg.board_member, sig_arg.nonce)
            }
            ItemToSign::Group(group_id) => {
                self.serialize_and_hash_group(group_id, &sig_arg.board_member, sig_arg.nonce)
            }
        };
        let signature_struct = Signature {
            signature_type: sig_arg.signature_type,
            raw_sig_bytes: sig_arg.raw_sig_bytes,
        };
        signature_struct
            .check_signature_by_type(&sig_arg.board_member, bytes_to_sign.as_managed_buffer());
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

        let action_type_encode_result = ActionType::SimpleAction.dep_encode(&mut all_data);
        require!(
            action_type_encode_result.is_ok(),
            ENCODING_ACTION_TYPE_ERR_MSG
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

        let action_type_encode_result = ActionType::Group.dep_encode(&mut all_data);
        require!(
            action_type_encode_result.is_ok(),
            ENCODING_ACTION_TYPE_ERR_MSG
        );

        self.crypto().sha256(all_data)
    }
}
