#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

pub mod bulk_payments_proxy;

const MAX_USERS_PER_TIER: usize = 1_000;
const MAX_TIERS: usize = 10;
const FOUR_HOURS: u64 = 60 * 60 * 4; // 4 hours

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode)]
pub struct UserAddrTimestamp<M: ManagedTypeApi> {
    pub addr: ManagedAddress<M>,
    pub timestamp: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait TieredBulkPayments {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(initialSetup)]
    fn initial_setup(&self) {}

    #[endpoint(optIn)]
    fn opt_in(&self) {
        let caller = self.blockchain().get_caller();

        require!(
            self.is_user_whitelisted(caller.clone()),
            "User not whitelisted"
        );
        for tier in 1..MAX_TIERS {
            let mut users_in_tier_mapper = self.opted_in_users_by_tier(tier);
            let tier_addresses_mapper = self.tier_addresses(tier);

            //If the contract is not deployed for the tier, deploy it
            if tier_addresses_mapper.is_empty() {
                let new_addr = self.deploy_bulk_payments_sc(tier);
                tier_addresses_mapper.set(new_addr);
            }
            // If the tier is full, go to the next one
            if users_in_tier_mapper.len() > MAX_USERS_PER_TIER {
                continue;
            }
            // Add user to tier
            users_in_tier_mapper.push(&UserAddrTimestamp {
                addr: caller,
                timestamp: self.get_current_time() + FOUR_HOURS,
            });
            break;
        }
    }

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
    #[endpoint(setBulkPaymentsScSourceAddress)]
    fn set_bulk_payments_sc_source_address(&self, sc_addr: ManagedAddress) {
        self.require_sc_address(&sc_addr);

        self.bulk_payments_sc_source_address().set(sc_addr);
    }

    #[only_owner]
    #[endpoint(deployBulkPaymentsSc)]
    fn deploy_bulk_payments_sc(&self, tier: usize) -> ManagedAddress {
        let bulk_payments_sc_source_address_mapper = self.bulk_payments_sc_source_address();
        require!(
            !bulk_payments_sc_source_address_mapper.is_empty(),
            "Template contract is not deployed"
        );
        let template_addr = bulk_payments_sc_source_address_mapper.get();

        let tier_addresses_mapper = self.tier_addresses(tier);
        require!(
            tier_addresses_mapper.is_empty(),
            "Already deployed contract for this tier"
        );

        let gas_left = self.blockchain().get_gas_left();
        let new_contract_address = self
            .tx()
            .raw_deploy()
            .arguments_raw(ManagedArgBuffer::new())
            .gas(gas_left)
            .from_source(template_addr.clone())
            .code_metadata(self.blockchain().get_code_metadata(&template_addr))
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.tier_addresses(tier).set(new_contract_address.clone());
        new_contract_address
    }

    #[payable("*")]
    #[only_owner]
    #[endpoint(distributePayments)]
    fn distribute_payments(&self, tier: usize) {
        let payment = self.call_value().egld_or_single_esdt();

        let opted_in_users_by_tier_mapper = self.opted_in_users_by_tier(tier);
        let users_iter = opted_in_users_by_tier_mapper.iter();

        let bulk_payments_addr = self.tier_addresses(tier).get();

        let mut multi_value_users: MultiValueEncoded<MultiValue2<ManagedAddress, u64>> =
            MultiValueEncoded::new();
        for user in users_iter {
            multi_value_users.push(MultiValue2::from((user.addr, user.timestamp)));
        }

        self.tx()
            .to(bulk_payments_addr)
            .typed(bulk_payments_proxy::BulkPaymentsProxy)
            .distribute_rewards(multi_value_users)
            .payment(payment)
            .sync_call();
    }

    // views

    #[view(getScAddrFromTier)]
    fn get_sc_addr_from_tier(&self, tier: usize) -> ManagedAddress {
        self.tier_addresses(tier).get()
    }

    #[view(isUserWhitelisted)]
    fn is_user_whitelisted(&self, user: ManagedAddress) -> bool {
        self.user_whitelist().contains(&user)
    }

    // internal functions

    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }

    fn require_whitelisted(&self, user: &ManagedAddress) {
        require!(
            self.user_whitelist().contains(user),
            "May not call this function"
        );
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    // Events
    #[event("performDeployFromSource")]
    fn perform_deploy_from_source_event(
        &self,
        #[indexed] egld_value: &BigUint,
        #[indexed] source_address: &ManagedAddress,
        #[indexed] code_metadata: CodeMetadata,
        #[indexed] gas: u64,
        #[indexed] arguments: &MultiValueManagedVec<ManagedBuffer>,
    );

    // Storages

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<ManagedAddress>;

    #[storage_mapper("tierAddresses")]
    fn tier_addresses(&self, tier: usize) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("optedInUsersByTier")]
    fn opted_in_users_by_tier(&self, tier: usize) -> VecMapper<UserAddrTimestamp<Self::Api>>;

    #[storage_mapper("bulkPaymentsScAddr")]
    fn bulk_payments_sc_source_address(&self) -> SingleValueMapper<ManagedAddress>;
}
