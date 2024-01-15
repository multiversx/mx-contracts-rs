#![no_std]

multiversx_sc::imports!();

pub mod token_info;
pub mod transfer;

#[multiversx_sc::contract]
pub trait FairLaunch: token_info::TokenInfoModule + transfer::TransferModule {
    #[init]
    fn init(&self) {}
}
