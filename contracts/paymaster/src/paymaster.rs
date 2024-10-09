#![no_std]

use multiversx_sc::imports::*;

pub mod forward_call;
pub mod paymaster_proxy;
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
        min_gas_limit: u64,
        endpoint_name: ManagedBuffer,
        endpoint_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let original_caller = self.blockchain().get_caller();
        let own_sc_address = self.blockchain().get_sc_address();
        let own_shard = self.blockchain().get_shard_of_address(&own_sc_address);
        let original_caller_shard = self.blockchain().get_shard_of_address(&original_caller);
        require!(
            own_shard == original_caller_shard,
            "Caller must be in the same shard"
        );

        let payments = self.call_value().all_esdt_transfers();
        require!(!payments.is_empty(), "There is no fee for payment!");

        let fee_payment = payments.get(FEE_PAYMENT_INDEX);
        self.tx()
            .to(&relayer_addr)
            .payment(EsdtTokenPayment::new(
                fee_payment.token_identifier,
                fee_payment.token_nonce,
                fee_payment.amount,
            ))
            .transfer();

        let mut payments_without_fee = payments.clone_value();
        payments_without_fee.remove(FEE_PAYMENT_INDEX);

        self.forward_call(
            dest,
            min_gas_limit,
            endpoint_name,
            payments_without_fee,
            endpoint_args,
        );
    }
}
