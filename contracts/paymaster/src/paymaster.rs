#![no_std]

multiversx_sc::imports!();

pub mod forward_call;
const FEE_PAYMENT: usize = 0;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait PaymasterContract: forward_call::ForwardCall {
    #[init]
    fn init(&self) {}

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
        
        let fee_payment = payments.get(0).clone();
        self.send().direct_esdt(
            &relayer_addr,
            &fee_payment.token_identifier,
            0,
            &fee_payment.amount,
        );

        let mut payments_without_fee = payments.clone_value();
        payments_without_fee.remove(FEE_PAYMENT);
        
        self.forward_call(dest, endpoint_name, endpoint_args, payments_without_fee);
    }
}
