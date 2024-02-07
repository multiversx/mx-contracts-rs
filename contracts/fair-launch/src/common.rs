multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
pub type Percentage = u32;

pub const MAX_FEE_PERCENTAGE: u32 = 10_000; // 100%

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct TakeFeesResult<M: ManagedTypeApi> {
    pub original_caller: ManagedAddress<M>,
    pub original_payments: PaymentsVec<M>,
    pub fees: PaymentsVec<M>,
    pub transfers: PaymentsVec<M>,
}

#[multiversx_sc::module]
pub trait CommonModule {
    fn get_non_empty_payments(&self) -> PaymentsVec<Self::Api> {
        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(!payments.is_empty(), "Empty payments");

        payments
    }

    fn take_fees(
        &self,
        caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
        fees_percentage: ManagedVec<Percentage>,
    ) -> TakeFeesResult<Self::Api> {
        if self.user_whitelist().contains(&caller) {
            return TakeFeesResult {
                original_caller: caller,
                original_payments: payments.clone(),
                fees: ManagedVec::new(),
                transfers: payments,
            };
        }

        let original_payments = payments.clone();
        let mut final_payments = PaymentsVec::new();
        let mut fees_payments = PaymentsVec::new();

        for (payment, token_fees_percentage) in payments.iter().zip(fees_percentage.iter()) {
            if token_fees_percentage == 0 {
                final_payments.push(payment);

                continue;
            }

            let fee_amount = self.calculate_fee_rounded_up(&payment.amount, token_fees_percentage);
            let user_payment = EsdtTokenPayment::new(
                payment.token_identifier.clone(),
                payment.token_nonce,
                &payment.amount - &fee_amount,
            );

            if fee_amount > 0 {
                let fee_payment = EsdtTokenPayment::new(
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    fee_amount,
                );
                fees_payments.push(fee_payment);
            }

            if user_payment.amount > 0 {
                final_payments.push(user_payment);
            }
        }

        TakeFeesResult {
            original_caller: caller,
            original_payments,
            fees: fees_payments,
            transfers: final_payments,
        }
    }

    fn calculate_fee_rounded_up(&self, payment_amount: &BigUint, fees_percentage: Percentage) -> BigUint {
        (payment_amount * fees_percentage + MAX_FEE_PERCENTAGE - 1u32) / MAX_FEE_PERCENTAGE
    }

    fn burn_all_tokens(&self, tokens: &PaymentsVec<Self::Api>) {
        for token in tokens {
            if token.amount > 0 {
                self.send().esdt_local_burn(
                    &token.token_identifier,
                    token.token_nonce,
                    &token.amount,
                );
            }
        }
    }

    #[view(getTokenFees)]
    #[storage_mapper("tokenFees")]
    fn token_fees(&self, token_id: &TokenIdentifier) -> SingleValueMapper<Percentage>;

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<Self::Api, ManagedAddress>;
}
