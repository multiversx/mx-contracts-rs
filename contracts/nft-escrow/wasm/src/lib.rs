// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            8
// Async Callback (empty):               1
// Total number of exported functions:  10

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    nft_escrow
    (
        init => init
        escrow => escrow
        cancel => cancel
        accept => accept
        getCreatedOffers => get_created_offers
        getWantedOffers => get_wanted_offers
        created_offers => created_offers
        wanted_offers => wanted_offers
        offers => offers
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
