#![no_std]

multiversx_sc::imports!();

pub mod fees;
pub mod forward_call;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait PaymasterContract: fees::FeesModule + forward_call::ForwardCall {
    #[init]
    fn init(&self, price_query_address: ManagedAddress) {
        self.price_query_address().set(price_query_address);
    }

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

        self.pay_fee_to_relayer(relayer_addr, payments.clone_value());
        let mut payments_without_fee = payments.clone_value();
        payments_without_fee.remove(0);
        self.forward_call(dest, endpoint_name, endpoint_args, payments_without_fee);
    }
}
