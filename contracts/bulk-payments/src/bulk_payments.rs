#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait BulkPayments {
    #[init]
    fn init(&self) {
        // self.current_batch_index().set(0usize);
        // self.max_batch_index().set(0usize);
        // self.batch_size().set(90usize);
    }

    #[upgrade]
    fn upgrade(&self, batch_size: usize, max_batch_index: usize) {
        // self.batch_size().set(batch_size);
        // self.max_batch_index().set(max_batch_index);
    }

    #[endpoint(joinParty)]
    fn join_party(&self) {
        let mut max_batch_index = self.max_batch_index().get();
        let batch_size = self.batch_size().get();

        let mut last_batch = self.batched_addresses(max_batch_index).get();

        let caller = self.blockchain().get_caller();

        if last_batch.len() < batch_size {
            // Last batch is not full; insert there
            last_batch.push(caller.clone());
            self.batched_addresses(max_batch_index).set(last_batch);
        } else {
            // Last batch is full; Create a new batch
            max_batch_index += 1;
            let mut new_batch = ManagedVec::new();
            new_batch.push(caller.clone());
            self.batched_addresses(max_batch_index).set(new_batch);
            self.max_batch_index().set(max_batch_index);
        }
        self.addr_joined_event(&caller, max_batch_index);
    }

    #[payable("*")]
    #[endpoint]
    fn bulksend(&self, payment_amount: BigUint) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();

        self.debug(token.clone(), payment.clone());

        let mut current_batch_index = self.current_batch_index().get();
        let max_batch_index = self.max_batch_index().get();

        if current_batch_index > max_batch_index {
            // Go through all addr from beginning
            current_batch_index = 0;
        }

        let current_batch = self.batched_addresses(current_batch_index).get();

        for destination in current_batch {
            self.tx()
                .to(destination)
                .egld_or_single_esdt(&token, 0, &payment_amount)
                .transfer();
        }
        current_batch_index += 1;
        self.current_batch_index().set(current_batch_index);
    }

    #[endpoint(join)]
    fn join(&self) {
        self.joined_addr().push(&self.blockchain().get_caller());
    }

    #[payable("*")]
    #[endpoint(sendRewards)]
    fn send_rewards(&self, payment_amount: BigUint) {
        let (token, _, _) = self.call_value().egld_or_single_esdt().into_tuple();
        let receivers_mapper = self.joined_addr();
        let receivers_iter = receivers_mapper.iter();
        for destination in receivers_iter {
            self.tx()
                .to(destination)
                .egld_or_single_esdt(&token, 0, &payment_amount)
                .transfer();
        }
    }

    // # view - check if user is opted-in

    #[view(getBatchFromIndex)]
    fn get_batch_from_index(&self, index: usize) -> ManagedVec<ManagedAddress> {
        self.batched_addresses(index).get()
    }

    #[view(currentBatchIndex)]
    #[storage_mapper("currentBatchIndex")]
    fn current_batch_index(&self) -> SingleValueMapper<usize>;

    #[view(maxBatchIndex)]
    #[storage_mapper("maxBatchIndex")]
    fn max_batch_index(&self) -> SingleValueMapper<usize>;

    #[view(batchSize)]
    #[storage_mapper("batchSize")]
    fn batch_size(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("batchedAddresses")]
    fn batched_addresses(&self, index: usize) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

    /// events
    #[event("addrJoined")]
    fn addr_joined_event(&self, #[indexed] user: &ManagedAddress, #[indexed] batch_id: usize);

    #[event("debug")]
    fn debug(&self, #[indexed] token_id: EgldOrEsdtTokenIdentifier, #[indexed] amount: BigUint);

    #[storage_mapper("joinedAddr")]
    fn joined_addr(&self) -> VecMapper<ManagedAddress>;
}
