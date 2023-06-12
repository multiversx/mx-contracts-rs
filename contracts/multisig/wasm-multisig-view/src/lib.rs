// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            8
// Async Callback (empty):               1
// Total number of exported functions:  10

#![no_std]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::external_view_endpoints! {
    multisig
    (
        getPendingActionFullInfo
        userRole
        getAllBoardMembers
        getAllProposers
        getActionData
        getActionSigners
        getActionSignerCount
        getActionValidSignerCount
    )
}

multiversx_sc_wasm_adapter::empty_callback! {}