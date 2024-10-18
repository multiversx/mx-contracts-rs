#![no_std]

#[allow(unused_imports)]
use multiversx_sc::{derive_imports::*, imports::*};

pub type Signature<M> = ManagedByteArray<M, ED25519_SIGNATURE_BYTE_LEN>;
pub const ED25519_SIGNATURE_BYTE_LEN: usize = 64;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Faucet {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

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
    fn claim(&self, signature: &Signature<Self::Api>) {
        let caller = self.blockchain().get_caller();
        let claimers_mapper = self.claimers();
        require!(!claimers_mapper.contains(&caller), "Caller already claimed");

        self.verify_signature(&caller, signature);
        self.claimers().add(&caller);

        let mut payments = ManagedVec::new();
        for payment in self.payments().iter() {
            payments.push(payment);
        }

        self.tx().to(ToCaller).with_multi_token_transfer(payments).transfer();
    }

    fn verify_signature(
        &self, 
        caller: &ManagedAddress<Self::Api>,
        signature: &Signature<Self::Api>,
    ) {
        let mut data = ManagedBuffer::new();
        let _ = caller.dep_encode(&mut data);

        let signer = self.signer().get();
        self.crypto().verify_ed25519(
            signer.as_managed_buffer(),
            &data,
            signature.as_managed_buffer(),
        );
    }

    #[only_owner]
    #[endpoint(setSigner)]
    fn set_signer(&self, signer: ManagedAddress<Self::Api>) {
        self.signer().set(&signer);
    }

    #[storage_mapper("payments")]
    fn payments(&self) -> VecMapper<EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress<Self::Api>>;

    #[storage_mapper("claimers")]
    fn claimers(&self) -> WhitelistMapper<ManagedAddress<Self::Api>>;
}
