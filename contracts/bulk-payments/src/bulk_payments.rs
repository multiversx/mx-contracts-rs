#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait BulkPayments {
    #[init]
    fn init(&self) {
        self.current_batch_index().set(0usize);
        self.max_batch_index().set(0usize);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(joinParty)]
    fn join_party(&self) {
        let mut max_batch_index = self.max_batch_index().get();
        let batch_size = self.batch_size().get();

        let mut last_batch = self.batched_addresses(max_batch_index).get();

        let caller = self.blockchain().get_caller();

        if batch_size < last_batch.len() {
            // Last batch is not full; insert there
            last_batch.push(caller);
            self.batched_addresses(max_batch_index).set(last_batch);
        } else {
            // Last batch is full; Create a new batch
            max_batch_index += 1;
            let mut new_batch = ManagedVec::new();
            new_batch.push(caller);
            self.batched_addresses(max_batch_index).set(new_batch);
        }
    }

    #[payable("*")]
    #[endpoint]
    fn bulksend(&self, payment_amount: BigUint) {
        let mut current_batch_index = self.current_batch_index().get();
        let max_batch_index = self.max_batch_index().get();
        let native_token = self.native_token().get();

        let payment = EsdtTokenPayment::new(native_token, 0u64, payment_amount);

        if current_batch_index > max_batch_index {
            // Go through all addr from beginning
            current_batch_index = 0;
        }

        let current_batch = self.batched_addresses(current_batch_index).get();

        for destination in current_batch {
            self.tx().to(destination).payment(&payment).transfer();
        }
        current_batch_index += 1;
        self.current_batch_index().set(current_batch_index);
    }

    #[storage_mapper("currentBatchIndex")]
    fn current_batch_index(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("maxBatchIndex")]
    fn max_batch_index(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("batchSize")]
    fn batch_size(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("batchedAddresses")]
    fn batched_addresses(&self, index: usize) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

    #[view(getNativeToken)]
    #[storage_mapper("nativeToken")]
    fn native_token(&self) -> SingleValueMapper<TokenIdentifier<Self::Api>>;
}
