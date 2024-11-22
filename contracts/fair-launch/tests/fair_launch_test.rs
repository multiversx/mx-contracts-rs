use crowdfunding_esdt::crowdfunding_esdt_proxy;
use fair_launch::{common::CommonModule, fair_launch_proxy, initial_launch::InitialLaunchModule};
use multiversx_sc::types::{ManagedAddress, ManagedBuffer, MultiValueEncoded, TestEsdtTransfer};
use multiversx_sc_scenario::{
    managed_biguint, managed_buffer, ExpectMessage, ScenarioTxRun, ScenarioTxWhitebox,
};
use tests_common::{
    FairLaunchSetup, BUY_FEE_PERCENTAGE_END, BUY_FEE_PERCENTAGE_START, CODE_PATH_CROWDFUNDING,
    CROWDFUNDING_ADDRESS, FAIR_LAUNCH_ADDRESS, FIRST_ADDRESS, OTHER_TOKEN_ID, OWNER,
    PAIR_MOCK_ADDRESS, SECOND_ADDRESS, TOKEN_ID,
};

mod tests_common;

#[test]
fn init_test() {
    let _ = FairLaunchSetup::new(None, 0);
}

#[test]
fn percentage_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);
    fl_setup.world.current_block().block_nonce(10);
    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .whitebox(fair_launch::contract_obj, |sc| {
            let percentage =
                sc.get_fee_percentage(BUY_FEE_PERCENTAGE_START, BUY_FEE_PERCENTAGE_END);
            let expected_percentage = BUY_FEE_PERCENTAGE_START - 808; // (BUY_FEE_PERCENTAGE_END - BUY_FEE_PERCENTAGE_START) * 10 blocks / (100 blocks - 1) ~= 808
            assert_eq!(percentage, expected_percentage);
        })
}

#[test]
fn calculate_fee_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);

    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .whitebox(fair_launch::contract_obj, |sc| {
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
}

#[test]
fn transfer_user_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);
    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .set_token_fees(TOKEN_ID, 4_000u32)
        .run();

    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1_000);

    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .forward_transfer(SECOND_ADDRESS, MultiValueEncoded::new())
        .esdt(TestEsdtTransfer(TOKEN_ID, 0, 1000))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 0);
    fl_setup
        .world
        .check_account(SECOND_ADDRESS)
        .esdt_balance(TOKEN_ID, 600);
    fl_setup
        .world
        .check_account(OWNER)
        .esdt_balance(TOKEN_ID, 400);
}

#[test]
fn transfer_sc_ok_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);
    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .set_token_fees(TOKEN_ID, 4000u32)
        .run();

    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1_000);

    fl_setup
        .world
        .tx()
        .from(OWNER)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .init(2000u32, 1000u32, TOKEN_ID)
        .new_address(CROWDFUNDING_ADDRESS)
        .code(CODE_PATH_CROWDFUNDING)
        .run();

    let mut args = MultiValueEncoded::new();
    args.push(managed_buffer!(b"fund"));
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .forward_transfer(CROWDFUNDING_ADDRESS, args)
        .esdt(TestEsdtTransfer(TOKEN_ID, 0, 1_000))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 0);
    fl_setup
        .world
        .check_account(CROWDFUNDING_ADDRESS)
        .esdt_balance(TOKEN_ID, 600);
    fl_setup
        .world
        .check_account(FAIR_LAUNCH_ADDRESS)
        .esdt_balance(TOKEN_ID, 400);
}

#[test]
fn transfer_sc_fail_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);
    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .set_token_fees(TOKEN_ID, 4000u32)
        .run();

    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1_000);

    fl_setup
        .world
        .tx()
        .from(OWNER)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .init(2000u32, 1000u32, TOKEN_ID)
        .new_address(CROWDFUNDING_ADDRESS)
        .code(CODE_PATH_CROWDFUNDING)
        .run();

    let mut args = MultiValueEncoded::new();
    args.push(managed_buffer!(b"claim"));
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .forward_transfer(CROWDFUNDING_ADDRESS, args)
        .esdt(TestEsdtTransfer(TOKEN_ID, 0, 0))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1000);
    fl_setup
        .world
        .check_account(CROWDFUNDING_ADDRESS)
        .esdt_balance(TOKEN_ID, 0);
    fl_setup
        .world
        .check_account(FAIR_LAUNCH_ADDRESS)
        .esdt_balance(TOKEN_ID, 0);
}

#[test]
fn transfer_whitelist_test() {
    let mut fl_setup = FairLaunchSetup::new(None, 0);
    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .set_token_fees(TOKEN_ID, 4000u32)
        .run();

    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1_000);

    let mut users = MultiValueEncoded::new();
    users.push(ManagedAddress::from_address(&FIRST_ADDRESS.to_address()));

    fl_setup
        .world
        .tx()
        .from(OWNER)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .add_users_to_whitelist(users)
        .run();

    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .forward_transfer(SECOND_ADDRESS, MultiValueEncoded::new())
        .esdt(TestEsdtTransfer(TOKEN_ID, 0, 1000))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 0);
    fl_setup
        .world
        .check_account(SECOND_ADDRESS)
        .esdt_balance(TOKEN_ID, 1000);
    fl_setup
        .world
        .check_account(OWNER)
        .esdt_balance(TOKEN_ID, 0);
}

#[test]
fn buy_token_test() {
    let mut fl_setup = FairLaunchSetup::new(Some(TOKEN_ID), 50);
    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 1000);

    // 90% of the initial value is kept as fee
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .buy_token(PAIR_MOCK_ADDRESS, 1u32)
        .esdt(TestEsdtTransfer(OTHER_TOKEN_ID, 0, 1000))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 50);
    fl_setup
        .world
        .check_account(FAIR_LAUNCH_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 900);
    fl_setup
        .world
        .check_account(PAIR_MOCK_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 100);
}

#[test]
fn sell_token_test() {
    let mut fl_setup = FairLaunchSetup::new(Some(OTHER_TOKEN_ID), 5000);
    fl_setup.world.current_block().block_nonce(50);
    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 1000);

    // 75% of the initial value is kept as fee
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .sell_token(PAIR_MOCK_ADDRESS, OTHER_TOKEN_ID, 1u32)
        .esdt(TestEsdtTransfer(TOKEN_ID, 0, 1000))
        .run();

    // values slightly differ due to biguint division errors
    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 504);
    fl_setup
        .world
        .check_account(FAIR_LAUNCH_ADDRESS)
        .esdt_balance(TOKEN_ID, 748);
    fl_setup
        .world
        .check_account(PAIR_MOCK_ADDRESS)
        .esdt_balance(TOKEN_ID, 252);
}

#[test]
fn try_buy_after_deadline() {
    let mut fl_setup = FairLaunchSetup::new(Some(TOKEN_ID), 50);
    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 1000);

    fl_setup.world.current_block().block_nonce(120);

    // 90% of the initial value is kept as fee
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .buy_token(PAIR_MOCK_ADDRESS, 1u32)
        .esdt(TestEsdtTransfer(OTHER_TOKEN_ID, 0, 1000))
        .with_result(ExpectMessage(
            "Cannot call this endpoint, initial launch period passed",
        ))
        .run();
}

#[test]
fn forward_swap_sync_test() {
    let mut fl_setup = FairLaunchSetup::new(Some(TOKEN_ID), 500_000);
    fl_setup.world.current_block().block_nonce(101);
    fl_setup
        .world
        .account(FIRST_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 100_000);

    let mut args = MultiValueEncoded::new();
    args.push(TOKEN_ID.to_token_identifier().into_managed_buffer());
    args.push(ManagedBuffer::from(b"0"));
    // 40% of the initial value is kept as fee
    fl_setup
        .world
        .tx()
        .from(FIRST_ADDRESS)
        .to(FAIR_LAUNCH_ADDRESS)
        .typed(fair_launch_proxy::FairLaunchProxy)
        .forward_execute_on_dest(PAIR_MOCK_ADDRESS, b"swapTokensFixedInput", args)
        .esdt(TestEsdtTransfer(OTHER_TOKEN_ID, 0, 100_000))
        .run();

    fl_setup
        .world
        .check_account(FIRST_ADDRESS)
        .esdt_balance(TOKEN_ID, 30_000);
    fl_setup
        .world
        .check_account(FAIR_LAUNCH_ADDRESS)
        .esdt_balance(OTHER_TOKEN_ID, 40_000);
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
