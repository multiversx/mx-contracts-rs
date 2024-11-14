#![no_std]

use multiversx_sc::imports::*;

pub mod available_tokens;
pub mod dn404_proxy;
pub mod fee;
pub mod price;

pub type Nonce = u64;
pub type Percentage = u32;

pub const MAX_PERCENTAGE: Percentage = 10_000;
pub const NFT_AMOUNT: u32 = 1;

#[multiversx_sc::contract]
pub trait Dn404:
    available_tokens::AvailableTokensModule
    + price::PriceModule
    + fee::FeeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    /// Needs mint and burn roles for fractal_token
    #[init]
    fn init(&self, fractal_token_id: TokenIdentifier, admins: MultiValueEncoded<ManagedAddress>) {
        require!(
            fractal_token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );

        self.fractal_token().set_token_id(fractal_token_id);

        let caller = self.blockchain().get_caller();
        let _ = self.admins().insert(caller);
        self.admins().extend(admins);
        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
