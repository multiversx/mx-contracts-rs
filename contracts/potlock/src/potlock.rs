#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();
pub mod potlock_admin_interactions;
pub mod potlock_interactions;
pub mod potlock_requirements;
pub mod potlock_storage;

#[multiversx_sc::contract]
pub trait Potlock:
    potlock_admin_interactions::PotlockAdminInteractions
    + potlock_interactions::PotlockInteractions
    + potlock_requirements::PotlockRequirements
    + potlock_storage::PotlockStorage
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self, admins: MultiValueEncoded<ManagedAddress>) {
        let caller = self.blockchain().get_caller();
        self.admins().insert(caller);
        for admin in admins {
            self.admins().insert(admin);
        }
    }

    #[upgrade]
    fn upgrade(&self) {}
}
