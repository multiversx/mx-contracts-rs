// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           12
// Async Callback (empty):               1
// Total number of exported functions:  14

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    on_chain_claim
    (
        init => init
        upgrade => upgrade
        claim => claim
        claimAndRepair => claim_and_repair
        updateState => update_state
        setRepairStreakTokenId => set_repair_streak_token_id
        getAddressInfo => get_address_info
        canBeRepaired => can_be_repaired
        getRepairStreakTokenIdentifier => repair_streak_token_identifier
        isAdmin => is_admin
        addAdmin => add_admin
        removeAdmin => remove_admin
        getAdmins => admins
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
