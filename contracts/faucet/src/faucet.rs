#![no_std]

#[allow(unused_imports)]
use multiversx_sc::{derive_imports::*, imports::*};

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Faucet {
    #[init]
    fn init(&self) {}

    #[only_owner]
    #[payable("*")]
    #[endpoint(deposit)]
    fn deposit(&self, max_users: u64) {
        let payments = self.call_value().all_esdt_transfers();
        for payment in payments.iter() {
            let amount_per_user = payment.amount / max_users;
            self.payments().push( &EsdtTokenPayment::new(payment.token_identifier, payment.token_nonce, amount_per_user));
        }
    }

    #[endpoint(claim)]
    fn claim(&self) {
        let mut payments = ManagedVec::new();
        for payment in self.payments().iter() {
            payments.push(payment);
        }

        self.tx().to(ToCaller).with_multi_token_transfer(payments).transfer();
    }

    #[storage_mapper("payments")]
    fn payments(&self) -> VecMapper<EsdtTokenPayment<Self::Api>>;

    #[upgrade]
    fn upgrade(&self) {}
}
