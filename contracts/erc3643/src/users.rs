multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UsersModule {
    #[only_owner]
    #[endpoint(addUsers)]
    fn add_users(&self, users: MultiValueEncoded<ManagedAddress>) {
        let mapper = self.user_whitelist();
        for user in users {
            mapper.add(&user);
        }
    }

    #[only_owner]
    #[endpoint(removeUsers)]
    fn remove_users(&self, users: MultiValueEncoded<ManagedAddress>) {
        let mapper = self.user_whitelist();
        for user in users {
            mapper.remove(&user);
        }
    }

    #[view(isUserWhitelisted)]
    fn is_user_whitelisted(&self, user: ManagedAddress) -> bool {
        self.user_whitelist().contains(&user)
    }

    fn require_whitelisted(&self, user: &ManagedAddress) {
        require!(
            self.user_whitelist().contains(user),
            "May not call this function"
        );
    }

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<ManagedAddress>;
}
