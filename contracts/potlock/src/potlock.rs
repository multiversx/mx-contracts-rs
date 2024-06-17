#![no_std]

use multiversx_sc::imports::*;
mod potlock_setup;
mod potlock_storage;
mod potlock_interactions;
mod potlock_admin_interactions;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Potlock: potlock_admin_interactions: PotlockAdminInteractions + potlock_interactions: PotlockInteractions + potlock_setup: PotlockSetup + potlock_storage: PotlockStorage {
    #[init]
    fn init(&self, admin: ManagedAddress) {
        self.admins().add(admin);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
