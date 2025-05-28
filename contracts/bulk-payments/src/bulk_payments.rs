#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

const MAX_USERS_ALLOW: usize = 1_000;
const _FOUR_HOURS: u64 = 60 * 60 * 4 * 1_000; // 4 hours
const FOUR_MINUTES: u64 = 4 * 60 * 1_000; // 4 minutes
const MAX_CLEANUP_ITER: usize = 100;

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, PartialEq)]
pub struct UserAddrTimestamp<M: ManagedTypeApi> {
    pub addr: ManagedAddress<M>,
    pub timestamp: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait BulkPayments {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(optIn)]
    fn opt_in(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.is_user_whitelisted(caller.clone()),
            "User not whitelisted"
        );

        require!(
            !self.is_user_opted_in(caller.clone()),
            "User already opted in"
        );

        require!(
            self.get_number_users_opted_in() < MAX_USERS_ALLOW,
            "Max CAP reached"
        );
        let deadline_timestamp = self.blockchain().get_block_timestamp() + FOUR_MINUTES;
        self.opted_in_users_addr_timestamp()
            .insert(UserAddrTimestamp {
                addr: caller.clone(),
                timestamp: deadline_timestamp,
            });
        self.opted_in_addrs().insert(caller);
    }

    // Move this offchain
    #[only_owner]
    #[payable("*")]
    #[endpoint(distribute)]
    fn distribute(&self) {
        let payment_amount = self.amount_to_send().get();
        let opted_in_users_addr_timestamp_mapper = self.opted_in_users_addr_timestamp();

        let current_timestamp = self.blockchain().get_block_timestamp();
        for user_addr_timestamp in opted_in_users_addr_timestamp_mapper.iter() {
            self.addr_feedback_event(
                &user_addr_timestamp.addr,
                user_addr_timestamp.timestamp,
                current_timestamp,
            );
            if user_addr_timestamp.timestamp > current_timestamp {
                // 1747660414308
                // 1747661136108
                self.tx()
                    .to(user_addr_timestamp.addr)
                    .egld(&payment_amount)
                    .transfer();
            }
        }
    }

    #[only_owner]
    #[endpoint(cleanupStorage)]
    fn cleanup_storage(&self) {
        let mut opted_in_users_addr_timestamp_mapper = self.opted_in_users_addr_timestamp();
        let mut opted_in_users_mapper = self.opted_in_addrs();

        let mut interations = 0usize;
        let mut index = 1usize;
        let current_timestamp: u64 = self.blockchain().get_block_timestamp();

        while interations < MAX_CLEANUP_ITER && index <= opted_in_users_addr_timestamp_mapper.len()
        {
            self.debug_count(interations, index);

            let user = opted_in_users_addr_timestamp_mapper.get_by_index(index);

            self.addr_feedback_event(&user.addr, user.timestamp, current_timestamp);

            if user.timestamp <= current_timestamp {
                opted_in_users_addr_timestamp_mapper.swap_remove(&user);
                opted_in_users_mapper.swap_remove(&user.addr);
            } else {
                index += 1;
            }
            interations += 1;
        }
    }

    #[only_owner]
    #[endpoint(whitelistUsers)]
    fn add_whitelist_users(&self, users: MultiValueEncoded<ManagedAddress>) {
        let whitelist_mapper = self.user_whitelist();
        for user in users {
            whitelist_mapper.add(&user);
        }
    }

    #[only_owner]
    #[endpoint(removeWhitelistWsers)]
    fn remove_whitelist_users(&self, users: MultiValueEncoded<ManagedAddress>) {
        let whitelist_mapper = self.user_whitelist();
        for user in users {
            whitelist_mapper.remove(&user);
        }
    }

    #[only_owner]
    #[endpoint(setAmountToSend)]
    fn set_amount_to_send(&self, amount: BigUint) {
        self.amount_to_send().set(amount);
    }

    /// # view - check if user is opted-in

    #[view(isUserWhitelisted)]
    fn is_user_whitelisted(&self, user: ManagedAddress) -> bool {
        self.user_whitelist().contains(&user)
    }

    #[view(isUserOptedIn)]
    fn is_user_opted_in(&self, user: ManagedAddress) -> bool {
        return self.opted_in_addrs().contains(&user);
    }

    // TODO: get_user_timestamp

    #[view(getUsersOptedIn)]
    fn get_users_opted_in(&self) -> MultiValueEncoded<ManagedAddress> {
        let mut users_opted_in = MultiValueEncoded::new();
        let opted_in_users_addr_timestamp_mapper = self.opted_in_users_addr_timestamp();
        let current_timestamp = self.blockchain().get_block_timestamp();

        for user_addr_timestamp in opted_in_users_addr_timestamp_mapper.iter() {
            if user_addr_timestamp.timestamp > current_timestamp {
                users_opted_in.push(user_addr_timestamp.addr);
            }
        }
        return users_opted_in;
    }

    #[view(getNumberUsersOptedIn)]
    fn get_number_users_opted_in(&self) -> usize {
        return self.opted_in_addrs().len();
    }

    #[storage_mapper("amountToSend")]
    fn amount_to_send(&self) -> SingleValueMapper<BigUint>;

    /// events
    #[event("addrJoined")]
    fn addr_feedback_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] timestamp: u64,
        #[indexed] current_timestamp: u64,
    );

    #[event("debugCount")]
    fn debug_count(&self, #[indexed] count: usize, #[indexed] storage_len: usize);

    #[storage_mapper("optedInUsersAddrTimestamp")]
    fn opted_in_users_addr_timestamp(&self) -> UnorderedSetMapper<UserAddrTimestamp<Self::Api>>;

    #[storage_mapper("optedInAddrs")]
    fn opted_in_addrs(&self) -> UnorderedSetMapper<ManagedAddress<Self::Api>>;

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<ManagedAddress>;
}
