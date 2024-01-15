#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod exchange_actions;
pub mod token_info;
pub mod transfer;

#[multiversx_sc::contract]
pub trait FairLaunch:
    common::CommonModule
    + exchange_actions::ExchangeActionsModule
    + token_info::TokenInfoModule
    + transfer::TransferModule
{
    #[init]
    fn init(&self) {}
}
