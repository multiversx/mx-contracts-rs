use multiversx_sc::api::{CryptoApi, CryptoApiImpl};

use super::action::{Action, GroupId, Nonce};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

/// Board member, internal user nonce, action type (SimpleAction or Batch), SignatureType (Ed25519, Secp256r1 or Secp256k1) and raw signature
pub type SignatureMultiArg<M> =
    MultiValue5<ManagedAddress<M>, Nonce, ActionType, SignatureType, ManagedBuffer<M>>;

pub struct DecomposeResultType<M: ManagedTypeApi> {
    pub board_member: ManagedAddress<M>,
    pub user_id: AddressId,
    pub nonce: Nonce,
    pub signature_type: SignatureType,
    pub raw_sig_bytes: ManagedBuffer<M>,
}

/// Note: Always add new signature types at the end, and NEVER delete any types.
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum SignatureType {
    Ed25519,
    Secp256r1,
    Secp256k1,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum ActionType {
    SimpleAction,
    Group,
}

pub enum ItemToSign<'a, M: ManagedTypeApi> {
    Action(&'a Action<M>),
    Group(GroupId),
}

impl ActionType {
    pub fn require_is_type<M: ManagedTypeApi>(&self, action_type: Self) {
        if !matches!(self, action_type) {
            M::error_api_impl().signal_error(b"Wrong action type signed");
        }
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Signature<M: ManagedTypeApi> {
    pub signature_type: SignatureType,
    pub raw_sig_bytes: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi + CryptoApi> Signature<M> {
    //#[cfg(not(debug_assertions))]
    pub fn check_signature_by_type(
        &self,
        user_address: &ManagedAddress<M>,
        bytes_to_sign: &ManagedBuffer<M>,
    ) {
        match self.signature_type {
            SignatureType::Ed25519 => M::crypto_api_impl().verify_ed25519_managed(
                user_address.as_managed_buffer().get_handle(),
                bytes_to_sign.get_handle(),
                self.raw_sig_bytes.get_handle(),
            ),
            SignatureType::Secp256r1 => todo!(), // not implemented yet
            SignatureType::Secp256k1 => {
                let verify_result = M::crypto_api_impl().verify_secp256k1_managed(
                    user_address.as_managed_buffer().get_handle(),
                    bytes_to_sign.get_handle(),
                    self.raw_sig_bytes.get_handle(),
                );
                if !verify_result {
                    M::error_api_impl().signal_error(b"Failed checking Secp256k1 signature");
                }
            }
        }
    }

    // #[cfg(debug_assertions)]
    // pub fn check_signature_by_type(
    //     &self,
    //     _user_address: &ManagedAddress<M>,
    //     _bytes_to_sign: &ManagedBuffer<M>,
    // ) {
    // }
}
