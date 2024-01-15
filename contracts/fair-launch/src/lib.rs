#![no_std]

multiversx_sc::imports!();

pub mod token_info;

#[multiversx_sc::contract]
pub trait FairLaunch: token_info::TokenInfoModule {
    #[init]
    fn init(&self) {}
}
