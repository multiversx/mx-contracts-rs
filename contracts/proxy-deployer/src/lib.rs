#![no_std]

multiversx_sc::imports!();

use multiversx_sc_modules::pause;

pub mod config;
pub mod contract_interactions;

#[multiversx_sc::contract]
pub trait ProxyDeployer:
    contract_interactions::ContractInteractionsModule + config::ConfigModule + pause::PauseModule
{
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn upgrade(&self) {}
}
