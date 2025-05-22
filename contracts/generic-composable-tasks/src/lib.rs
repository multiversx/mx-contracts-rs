#![no_std]
#![allow(deprecated)]

multiversx_sc::imports!();

pub mod call_dispatcher;
pub mod high_level_calls;
pub mod raw_call;
pub mod unique_payments;

#[multiversx_sc::contract]
pub trait GenericComposableTasks:
    raw_call::simple_transfer::SimpleTransferModule
    + raw_call::sync_call::SyncCallModule
    + raw_call::async_call::AsyncCallModule
    + raw_call::common::CommonModule
    + high_level_calls::HighLevelCallsModule
    + call_dispatcher::CallDispatcherModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
