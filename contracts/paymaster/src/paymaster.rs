#![no_std]

use multiversx_sc::imports::*;

pub mod forward_call;
pub mod paymaster_proxy;
const FEE_PAYMENT: usize = 0;

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

        let fee_payment = payments.get(FEE_PAYMENT);
        self.tx()
            .to(&relayer_addr)
            .payment(EsdtTokenPayment::new(
                fee_payment.token_identifier,
                0,
                fee_payment.amount,
            ))
            .transfer();

        let mut payments_without_fee = payments.clone_value();
        payments_without_fee.remove(FEE_PAYMENT);

        self.forward_call(dest, endpoint_name, payments_without_fee, endpoint_args);
    }
}
