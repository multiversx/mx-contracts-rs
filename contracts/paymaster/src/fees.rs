use crate::forward_call::PaymentsVec;

multiversx_sc::imports!();

pub mod pair_proxy {
    #[multiversx_sc::proxy]
    pub trait PairProxy {
        #[view(getSafePriceByDefaultOffset)]
        fn get_safe_price_by_default_offset(
            &self,
            pair_address: ManagedAddress,
            input_payment: EsdtTokenPayment,
        ) -> EsdtTokenPayment;
    }
}

#[multiversx_sc::module]
pub trait FeesModule {
    #[only_owner]
    #[endpoint(addAcceptedFeesTokens)]
    fn add_accepted_fees_tokens(
        &self,
        accepted_tokens: MultiValueEncoded<MultiValue2<TokenIdentifier, ManagedAddress>>,
    ) {
        for pair in accepted_tokens {
            let (token_id, pair_address) = pair.into_tuple();
            require!(token_id.is_valid_esdt_identifier(), "Invalid token");

            self.pair_address_for_token(&token_id).set(pair_address);
        }
    }

    fn get_price(&self, token_id: TokenIdentifier, amount: BigUint) -> BigUint {
        let mapper = self.pair_address_for_token(&token_id);
        require!(
            !mapper.is_empty(),
            "There is no pair addres for the token provided!"
        );

        let pair_address = mapper.get();
        let price_query_address = self.price_query_address().get();
        let price: EsdtTokenPayment = self
            .pair_proxy(price_query_address)
            .get_safe_price_by_default_offset(
                pair_address,
                EsdtTokenPayment::new(token_id, 0, amount),
            )
            .execute_on_dest_context();

        price.amount
    }

    fn pay_fee_to_relayer(&self, relayer_addr: ManagedAddress, payments: PaymentsVec<Self::Api>) {
        let mut payments_iter = payments.iter();
        let fee_payment = match payments_iter.next() {
            Some(fee) => fee,
            None => sc_panic!("Fee payment is missing!"),
        };

        self.send().direct_esdt(
            &relayer_addr,
            &fee_payment.token_identifier,
            0,
            &fee_payment.amount,
        );
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair_proxy::Proxy<Self::Api>;

    #[storage_mapper("pairAddressForToken")]
    fn pair_address_for_token(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("priceQueryAddress")]
    fn price_query_address(&self) -> SingleValueMapper<ManagedAddress>;
}
