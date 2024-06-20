#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();
pub mod potlock_admin_interactions;
pub mod potlock_interactions;
pub mod potlock_setup;
pub mod potlock_storage;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Potlock:
    potlock_admin_interactions::PotlockAdminInteractions
    + potlock_interactions::PotlockInteractions
    + potlock_setup::PotlockSetup
    + potlock_storage::PotlockStorage
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self, admin: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        self.admins().insert(caller);
        self.admins().insert(admin);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
