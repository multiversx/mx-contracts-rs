multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config;

#[multiversx_sc::module]
pub trait TokenAttributesModule: config::ConfigModule {
    fn create_new_tokens<T: TopEncode + NestedEncode>(
        &self,
        amount: BigUint,
        attributes: &T,
    ) -> EsdtTokenPayment {
        let mystery_box_token_mapper = self.mystery_box_token();
        let mut encoded_attributes = ManagedBuffer::new();
        attributes
            .dep_encode(&mut encoded_attributes)
            .unwrap_or_else(|err| sc_panic!(err.message_str()));

        let attributes_to_nonce_mapper = self.attributes_to_nonce_mapping(&encoded_attributes);
        let existing_nonce = attributes_to_nonce_mapper.get();
        if existing_nonce != 0 {
            return mystery_box_token_mapper.nft_add_quantity(existing_nonce, amount);
        }

        // We use the manual esdt_nft_create function instead of the NonFungibleTokenMapper's nft_create function,
        // as we need to also set the uri, option which is not available in the mapper's built in functions
        let mystery_box_token_id = mystery_box_token_mapper.get_token_id();
        let mystery_box_uris = self.mystery_box_uris().get();
        let empty_buffer = ManagedBuffer::new();
        let new_nonce = self.send().esdt_nft_create(
            &mystery_box_token_id,
            &amount,
            &empty_buffer,
            &BigUint::zero(),
            &empty_buffer,
            attributes,
            &mystery_box_uris,
        );
        attributes_to_nonce_mapper.set(new_nonce);

        EsdtTokenPayment::new(mystery_box_token_id, new_nonce, amount)
    }

    #[storage_mapper("attributesToNonceMapping")]
    fn attributes_to_nonce_mapping(&self, attributes: &ManagedBuffer) -> SingleValueMapper<u64>;
}
