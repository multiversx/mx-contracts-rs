#![no_std]

multiversx_sc::imports!();

pub mod forward_call;
const FEE_PAYMENT_INDEX: usize = 0;

#[multiversx_sc::contract]
pub trait PaymasterContract: forward_call::ForwardCall {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(forwardExecution)]
    #[payable("*")]
    fn forward_execution(
        &self,
        relayer_addr: ManagedAddress,
        dest: ManagedAddress,
        endpoint_name: ManagedBuffer,
        endpoint_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let payments = self.call_value().all_esdt_transfers();
        require!(!payments.is_empty(), "There is no fee for payment!");

        let fee_payment = payments.get(FEE_PAYMENT_INDEX);
        require!(
            fee_payment.token_nonce == 0,
            "Only fungible tokens are accepted as fee payments!"
        );

        self.send().direct_esdt(
            &relayer_addr,
            &fee_payment.token_identifier,
            fee_payment.token_nonce,
            &fee_payment.amount,
        );

        let mut payments_without_fee = payments.clone_value();
        payments_without_fee.remove(FEE_PAYMENT_INDEX);

        self.forward_call(dest, endpoint_name, payments_without_fee, endpoint_args);
    }
}
