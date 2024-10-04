// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           12
// Async Callback (empty):               1
// Total number of exported functions:  15

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    on_chain_claim
    (
        init => init
        upgrade => upgrade
        claim => claim
        reset => reset
        claimAndRepair => claim_and_repair
        updateState => update_state
        setRepairStreakPayment => set_repair_streak_payment
        getAddressInfo => get_address_info
        canBeRepaired => can_be_repaired
        getRepairStreakPayment => repair_streak_payment
        isAdmin => is_admin
        addAdmin => add_admin
        removeAdmin => remove_admin
        getAdmins => admins
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
