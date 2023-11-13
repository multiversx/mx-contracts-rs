#![no_std]

multiversx_sc::imports!();

pub mod address_to_id_mapper;
pub mod config;
pub mod contract_interactions;

#[multiversx_sc::contract]
pub trait ProxyDeployer:
    contract_interactions::ContractInteractionsModule + config::ConfigModule
{
    #[init]
    fn init(&self) {}

    #[endpoint]
    fn upgrade(&self) {}
}
