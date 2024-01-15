// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           12
// Async Callback:                       1
// Total number of exported functions:  14

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    fair_launch
    (
        init => init
        upgrade => upgrade
        getTokenFees => token_fees
        addExchangeEndpoint => add_exchange_endpoint
        removeExchangeEndpoint => remove_exchange_endpoint
        forwardExecuteOnDest => forward_execute_on_dest
        forwardAsyncCall => forward_async_call
        issueToken => issue_token
        setTransferRole => set_transfer_role
        setTokenFees => set_token_fees
        addUsersToWhitelist => add_users_to_whitelist
        removeUsersFromWhitelist => remove_users_from_whitelist
        forwardTransfer => forward_transfer
    )
}

multiversx_sc_wasm_adapter::async_callback! { fair_launch }
