use crate::{fee::FeeType, Nonce, MAX_PERCENTAGE, NFT_AMOUNT};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct TokenIdNoncePair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: Nonce,
}

#[multiversx_sc::module]
pub trait AvailableTokensModule:
    crate::price::PriceModule
    + crate::fee::FeeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {
        let payments = self.call_value().all_esdt_transfers().clone_value();
        let mut basket = self.basket_of_goods();
        for payment in &payments {
            self.add_tokens(&mut basket, payment);
        }
    }

    #[payable("*")]
    #[endpoint(depositBasketOfGoods)]
    fn deposit_basket_of_goods(&self) {
        self.require_not_paused();

        let payments = self.call_value().all_esdt_transfers().clone_value();
        let token_mapper = self.fractal_token();
        let token_id = token_mapper.get_token_id_ref();

        let mut basket = self.basket_of_goods();
        let mut total_output_payment = BigUint::zero();
        for payment in &payments {
            let price = self.try_get_price(&payment.token_identifier, payment.token_nonce);
            let mut price_as_payment = EsdtTokenPayment::new(token_id.clone(), 0, price);

            let fee_for_token = self.get_fee(&payment.token_identifier, payment.token_nonce);
            let fee_amount = match fee_for_token {
                FeeType::FixedAmount(amt) => amt,
                FeeType::Percentage(percentage) => {
                    &price_as_payment.amount * percentage / MAX_PERCENTAGE
                }
            };
            require!(price_as_payment.amount >= fee_amount, "Invalid state");

            price_as_payment.amount -= fee_amount;
            total_output_payment += &price_as_payment.amount;
            self.add_tokens(&mut basket, payment);
        }

        if total_output_payment > 0 {
            let caller = self.blockchain().get_caller();
            let _ = token_mapper.mint_and_send(&caller, total_output_payment);
        }
    }

    #[payable("*")]
    #[endpoint(claimBasketOfGoods)]
    fn claim_basket_of_goods(
        &self,
        tokens_to_claim: MultiValueEncoded<EsdtTokenPaymentMultiValue>,
    ) {
        self.require_not_paused();
        require!(!tokens_to_claim.is_empty(), "No tokens to claim");

        let token_mapper = self.fractal_token();
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        token_mapper.require_same_token(&payment_token);

        let mut total_cost = BigUint::zero();
        let mut tokens_vec = ManagedVec::<Self::Api, _>::new();
        let mut basket = self.basket_of_goods();
        for token_to_claim in tokens_to_claim {
            let token_as_payment = token_to_claim.into_esdt_token_payment();
            let price = self.try_get_price(
                &token_as_payment.token_identifier,
                token_as_payment.token_nonce,
            );
            total_cost += price;

            self.remove_token(
                &mut basket,
                token_as_payment.token_identifier.clone(),
                token_as_payment.token_nonce,
            );

            tokens_vec.push(token_as_payment);
        }

        require!(payment_amount >= total_cost, "Not enough tokens");

        token_mapper.burn(&total_cost);

        let remaining_tokens = payment_amount - total_cost;
        let remaining_tokens_payment = EsdtTokenPayment::new(payment_token, 0, remaining_tokens);
        self.tx()
            .to(ToCaller)
            .payment(&remaining_tokens_payment)
            .transfer();
        self.tx().to(ToCaller).payment(&tokens_vec).transfer();
    }

    fn add_tokens(
        &self,
        mapper: &mut UnorderedSetMapper<TokenIdNoncePair<Self::Api>>,
        payment: EsdtTokenPayment,
    ) {
        self.remaining_tokens(&payment.token_identifier, payment.token_nonce)
            .update(|amt| *amt += payment.amount);
        let _ = mapper.insert(TokenIdNoncePair {
            token_id: payment.token_identifier,
            nonce: payment.token_nonce,
        });
    }

    fn remove_token(
        &self,
        mapper: &mut UnorderedSetMapper<TokenIdNoncePair<Self::Api>>,
        token_id: TokenIdentifier,
        nonce: Nonce,
    ) {
        let remove = self.remaining_tokens(&token_id, nonce).update(|amt| {
            require!(*amt >= NFT_AMOUNT, "Not enough tokens to claim");

            *amt -= NFT_AMOUNT;
            *amt == 0
        });
        if remove {
            let _ = mapper.swap_remove(&TokenIdNoncePair { token_id, nonce });
        }
    }

    #[view(getBasketOfGoods)]
    #[storage_mapper("basketOfGoods")]
    fn basket_of_goods(&self) -> UnorderedSetMapper<TokenIdNoncePair<Self::Api>>;

    #[view(getRemainingTokens)]
    #[storage_mapper("remTok")]
    fn remaining_tokens(
        &self,
        token_id: &TokenIdentifier,
        nonce: Nonce,
    ) -> SingleValueMapper<BigUint>;
}
