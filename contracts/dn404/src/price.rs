use crate::Nonce;

use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait PriceModule {
    #[only_owner]
    #[endpoint(setInternalPriceForToken)]
    fn set_internal_price_for_token(
        &self,
        token_id: TokenIdentifier,
        nonce: Nonce,
        price: BigUint,
    ) {
        self.price_for_token(&token_id, nonce).set(price);
    }

    #[only_owner]
    #[endpoint(setInternalPriceForCollection)]
    fn set_internal_price_for_collection(&self, token_id: TokenIdentifier, price: BigUint) {
        self.price_for_collection(&token_id).set(price);
    }

    #[view(getPriceForToken)]
    fn try_get_price(&self, token_id: &TokenIdentifier, nonce: Nonce) -> BigUint {
        let price_for_token = self.price_for_token(token_id, nonce).get();
        if price_for_token > 0 {
            return price_for_token;
        }

        let price_for_collection = self.price_for_collection(token_id).get();
        require!(price_for_collection > 0, "No price set for token");

        price_for_collection
    }

    #[view(getFractalTokenId)]
    #[storage_mapper("fractalToken")]
    fn fractal_token(&self) -> FungibleTokenMapper;

    #[storage_mapper("priceColl")]
    fn price_for_collection(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("priceTok")]
    fn price_for_token(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<BigUint>;
}
