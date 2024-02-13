#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Erc3643 {
    #[init]
    fn init(&self) {}
}
