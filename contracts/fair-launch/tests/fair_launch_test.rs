#![allow(deprecated)]

mod tests_common;

use crowdfunding_esdt::Crowdfunding;
use fair_launch::{
    common::CommonModule, exchange_actions::ExchangeActionsModule,
    initial_launch::InitialLaunchModule, transfer::TransferModule,
};
use multiversx_sc::types::{ManagedBuffer, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, managed_token_id_wrapped,
    rust_biguint,
};
use tests_common::*;

#[test]
fn init_test() {
    let _ = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
}

#[test]
fn percentage_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup.b_mock.set_block_nonce(10);
    fl_setup
        .b_mock
        .execute_query(&fl_setup.fl_wrapper, |sc| {
            let percentage =
                sc.get_fee_percentage(BUY_FEE_PERCENTAGE_START, BUY_FEE_PERCENTAGE_END);
            let expected_percentage = BUY_FEE_PERCENTAGE_START - 808; // (BUY_FEE_PERCENTAGE_END - BUY_FEE_PERCENTAGE_START) * 10 blocks / (100 blocks - 1) ~= 808
            assert_eq!(percentage, expected_percentage);
        })
        .assert_ok();
}

#[test]
fn calculate_fee_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup
        .b_mock
        .execute_query(&fl_setup.fl_wrapper, |sc| {
            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1_000), 4_000);
            let expected_fee = managed_biguint!(400);
            assert_eq!(fee, expected_fee);

            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1), 4_000);
            let expected_fee = managed_biguint!(1);
            assert_eq!(fee, expected_fee);

            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1_001), 4_000);
            let expected_fee = managed_biguint!(401);
            assert_eq!(fee, expected_fee);
        })
        .assert_ok();
}

#[test]
fn transfer_user_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &fl_setup.fl_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_token_fees(managed_token_id!(TOKEN_ID), 4_000);
            },
        )
        .assert_ok();

    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.forward_transfer(
                    managed_address!(&fl_setup.second_user_address),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.first_user_address, TOKEN_ID, &rust_biguint!(0));

    fl_setup.b_mock.check_esdt_balance(
        &fl_setup.second_user_address,
        TOKEN_ID,
        &rust_biguint!(600),
    );

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.owner_address, TOKEN_ID, &rust_biguint!(400));
}

#[test]
fn transfer_sc_ok_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &fl_setup.fl_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_token_fees(managed_token_id!(TOKEN_ID), 4_000);
            },
        )
        .assert_ok();

    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    let cf_wrapper = fl_setup.b_mock.create_sc_account(
        &rust_biguint!(0),
        Some(&fl_setup.owner_address),
        crowdfunding_esdt::contract_obj,
        "cf wasm path",
    );
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &cf_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.init(
                    managed_biguint!(2_000),
                    1_000,
                    managed_token_id_wrapped!(TOKEN_ID),
                );
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                let mut args = MultiValueEncoded::new();
                args.push(ManagedBuffer::from(b"fund"));

                sc.forward_transfer(managed_address!(cf_wrapper.address_ref()), args);
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.first_user_address, TOKEN_ID, &rust_biguint!(0));

    fl_setup
        .b_mock
        .check_esdt_balance(cf_wrapper.address_ref(), TOKEN_ID, &rust_biguint!(600));

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.fl_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(400),
    );
}

#[test]
fn transfer_sc_fail_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &fl_setup.fl_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_token_fees(managed_token_id!(TOKEN_ID), 4_000);
            },
        )
        .assert_ok();

    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    let cf_wrapper = fl_setup.b_mock.create_sc_account(
        &rust_biguint!(0),
        Some(&fl_setup.owner_address),
        crowdfunding_esdt::contract_obj,
        "cf wasm path",
    );
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &cf_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.init(
                    managed_biguint!(2_000),
                    1_000,
                    managed_token_id_wrapped!(TOKEN_ID),
                );
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(0),
            |sc| {
                let mut args = MultiValueEncoded::new();
                args.push(ManagedBuffer::from(b"claim"));

                sc.forward_transfer(managed_address!(cf_wrapper.address_ref()), args);
            },
        )
        .assert_ok();

    fl_setup.b_mock.check_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    fl_setup
        .b_mock
        .check_esdt_balance(cf_wrapper.address_ref(), TOKEN_ID, &rust_biguint!(0));

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.owner_address, TOKEN_ID, &rust_biguint!(0));
}

#[test]
fn transfer_whitelist_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.owner_address,
            &fl_setup.fl_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_token_fees(managed_token_id!(TOKEN_ID), 4_000);
            },
        )
        .assert_ok();

    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    fl_setup
        .b_mock
        .execute_tx(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut users = MultiValueEncoded::new();
                users.push(managed_address!(&fl_setup.first_user_address));
                sc.add_users_to_whitelist(users);
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.forward_transfer(
                    managed_address!(&fl_setup.second_user_address),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.first_user_address, TOKEN_ID, &rust_biguint!(0));

    fl_setup.b_mock.check_esdt_balance(
        &fl_setup.second_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.owner_address, TOKEN_ID, &rust_biguint!(0));
}

#[test]
fn buy_token_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        OTHER_TOKEN_ID,
        &rust_biguint!(1_000),
    );
    fl_setup.b_mock.set_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(50),
    );

    // 90% of the initial value is kept as fee
    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            OTHER_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.buy_token(
                    managed_address!(fl_setup.pair_wrapper.address_ref()),
                    managed_biguint!(1u32),
                );
            },
        )
        .assert_ok();

    fl_setup
        .b_mock
        .check_esdt_balance(&fl_setup.first_user_address, TOKEN_ID, &rust_biguint!(50));

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.fl_wrapper.address_ref(),
        OTHER_TOKEN_ID,
        &rust_biguint!(900),
    );

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        OTHER_TOKEN_ID,
        &rust_biguint!(100),
    );
}

#[test]
fn sell_token_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup.b_mock.set_block_nonce(50);
    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(1_000),
    );
    fl_setup.b_mock.set_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        OTHER_TOKEN_ID,
        &rust_biguint!(5000),
    );

    // 75% of the initial value is kept as fee
    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.sell_token(
                    managed_address!(fl_setup.pair_wrapper.address_ref()),
                    managed_token_id!(OTHER_TOKEN_ID),
                    managed_biguint!(1u32),
                );
            },
        )
        .assert_ok();

    // values slightly differ due to biguint division errors

    fl_setup.b_mock.check_esdt_balance(
        &fl_setup.first_user_address,
        OTHER_TOKEN_ID,
        &rust_biguint!(504),
    );

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.fl_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(748),
    );

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(252),
    );
}

#[test]
fn try_buy_after_deadline() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        OTHER_TOKEN_ID,
        &rust_biguint!(1_000),
    );
    fl_setup.b_mock.set_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(50),
    );

    fl_setup.b_mock.set_block_nonce(120);

    // 90% of the initial value is kept as fee
    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            OTHER_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.buy_token(
                    managed_address!(fl_setup.pair_wrapper.address_ref()),
                    managed_biguint!(1u32),
                );
            },
        )
        .assert_user_error("Cannot call this endpoint, initial launch period passed");
}

#[test]
fn forward_swap_sync_test() {
    let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
    fl_setup.b_mock.set_block_nonce(101);
    fl_setup.b_mock.set_esdt_balance(
        &fl_setup.first_user_address,
        OTHER_TOKEN_ID,
        &rust_biguint!(100_000),
    );
    fl_setup.b_mock.set_esdt_balance(
        fl_setup.pair_wrapper.address_ref(),
        TOKEN_ID,
        &rust_biguint!(500_000),
    );

    // 40% of the initial value is kept as fee
    fl_setup
        .b_mock
        .execute_esdt_transfer(
            &fl_setup.first_user_address,
            &fl_setup.fl_wrapper,
            OTHER_TOKEN_ID,
            0,
            &rust_biguint!(100_000),
            |sc| {
                let mut args = MultiValueEncoded::new();
                args.push(managed_buffer!(TOKEN_ID));
                args.push(managed_buffer!(b"0"));

                sc.forward_execute_on_dest(
                    managed_address!(fl_setup.pair_wrapper.address_ref()),
                    managed_buffer!(b"swapTokensFixedInput"),
                    args,
                );
            },
        )
        .assert_ok();

    fl_setup.b_mock.check_esdt_balance(
        &fl_setup.first_user_address,
        TOKEN_ID,
        &rust_biguint!(30_000),
    );

    fl_setup.b_mock.check_esdt_balance(
        fl_setup.fl_wrapper.address_ref(),
        OTHER_TOKEN_ID,
        &rust_biguint!(40_000),
    );
}

// #[test]
// fn forward_swap_async_test() {
//     let mut fl_setup = FairLaunchSetup::new(fair_launch::contract_obj, pair_mock::contract_obj);
//     fl_setup.b_mock.set_block_nonce(101);
//     fl_setup.b_mock.set_esdt_balance(
//         &fl_setup.first_user_address,
//         OTHER_TOKEN_ID,
//         &rust_biguint!(100_000),
//     );
//     fl_setup.b_mock.set_esdt_balance(
//         fl_setup.pair_wrapper.address_ref(),
//         TOKEN_ID,
//         &rust_biguint!(500_000),
//     );

//     // 40% of the initial value is kept as fee
//     fl_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &fl_setup.first_user_address,
//             &fl_setup.fl_wrapper,
//             OTHER_TOKEN_ID,
//             0,
//             &rust_biguint!(100_000),
//             |sc| {
//                 let mut args = MultiValueEncoded::new();
//                 args.push(managed_buffer!(TOKEN_ID));
//                 args.push(managed_buffer!(b"0"));

//                 sc.forward_async_call(
//                     managed_address!(fl_setup.pair_wrapper.address_ref()),
//                     managed_buffer!(b"swapTokensFixedInput"),
//                     args,
//                 );
//             },
//         )
//         .assert_ok();

//     fl_setup
//         .b_mock
//         .check_esdt_balance(&fl_setup.first_user_address, TOKEN_ID, &rust_biguint!(30_000));

//     fl_setup.b_mock.check_esdt_balance(
//         fl_setup.fl_wrapper.address_ref(),
//         OTHER_TOKEN_ID,
//         &rust_biguint!(40_000),
//     );
// }
