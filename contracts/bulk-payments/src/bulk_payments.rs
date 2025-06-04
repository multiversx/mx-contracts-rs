#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

const MAX_USERS_ALLOW: usize = 1000;
const _FOUR_HOURS: u64 = 60 * 60 * 4 * 1_000; // 4 hours
const FOUR_MINUTES: u64 = 4 * 60 * 1_000; // 4 minutes
                                          // const MAX_CLEANUP_ITER: usize = 100;

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, PartialEq)]
pub struct UserAddrTimestamp<M: ManagedTypeApi> {
    pub addr: ManagedAddress<M>,
    pub timestamp: u64,
}

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

        let user_already_opted_in = self.is_user_opted_in(caller.clone());
        require!(
            user_already_opted_in && self.is_timestamp_expired(caller.clone()),
            "User already opted in"
        );

        let number_users_opted_in = self.get_number_users_opted_in();
        let timestamp = self.blockchain().get_block_timestamp();
        
        if number_users_opted_in >= MAX_USERS_ALLOW {
            require!(!user_already_opted_in, "Max cap reached");
            self.try_clear_first_user_if_timestamp_expired(caller.clone(), timestamp);
        }

        let deadline_timestamp = timestamp + FOUR_MINUTES;
        self.addr_timestamp(caller.clone()).set(deadline_timestamp);
        self.opted_in_addrs().insert(caller);
    }

    fn try_clear_first_user_if_timestamp_expired(&self, user: ManagedAddress, timestamp: u64) {
        let addr_timestamp = self.addr_timestamp(user.clone()).get();

        if addr_timestamp < timestamp {
            self.addr_timestamp(user.clone()).clear();
            self.opted_in_addrs().remove(&user);
        } else {
            sc_panic!("Max CAP reached");
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

    /// # views
    #[view(isUserWhitelisted)]
    fn is_user_whitelisted(&self, user: ManagedAddress) -> bool {
        self.user_whitelist().contains(&user)
    }

    #[view(isUserOptedIn)]
    fn is_user_opted_in(&self, user: ManagedAddress) -> bool {
        return self.opted_in_addrs().contains(&user);
    }

    #[view(isTimestampExpired)]
    fn is_timestamp_expired(&self, user: ManagedAddress) -> bool {
        let user_timestamp = self.addr_timestamp(user).get();

        user_timestamp > self.blockchain().get_block_timestamp()
    }

    // Returns user without timestamp expired
    #[view(getUsersOptedIn)]
    fn get_users_opted_in(&self) -> MultiValueEncoded<ManagedAddress> {
        let mut users_opted_in = MultiValueEncoded::new();
        let addr_timestamp_mapper = self.opted_in_addrs();
        let current_timestamp = self.blockchain().get_block_timestamp();

        for user_addr in addr_timestamp_mapper.iter() {
            let timestamp = self.addr_timestamp(user_addr.clone()).get();
            if timestamp > current_timestamp {
                users_opted_in.push(user_addr);
            }
        }
        return users_opted_in;
    }

    // Total user without timestamp expired
    #[view(getEligibleNumberUsersOptedIn)]
    fn get_eligible_number_users_opted_in(&self) -> usize {
        return self.get_users_opted_in().len();
    }

    // Total user with timestamp expired and not expired
    #[view(getNumberUsersOptedIn)]
    fn get_number_users_opted_in(&self) -> usize {
        return self.opted_in_addrs().len();
    }

    #[view(getUserTimestamp)]
    fn get_user_timstamp(&self, user: ManagedAddress) -> u64 {
        return self.addr_timestamp(user).get();
    }

    #[storage_mapper("addrTimestamp")]
    fn addr_timestamp(&self, user: ManagedAddress) -> SingleValueMapper<u64>;

    #[storage_mapper("optedInAddrs")]
    fn opted_in_addrs(&self) -> SetMapper<ManagedAddress<Self::Api>>;

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<ManagedAddress>;
}
