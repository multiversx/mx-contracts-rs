use generic_composable_tasks::call_dispatcher::CallDispatcherModule;
use generic_composable_tasks_test_setup::{
    build_egld_simple_transfer_data, build_esdt_simple_transfer_data,
    build_sync_call_egld_transfer_data, build_sync_call_esdt_transfer_data, GenericCompTasksSetup,
    RAND_ESDT_TOKEN_ID, UNWRAP_EGLD_ENDPOINT_NAME, USER_BALANCE, WEGLD_TOKEN_ID,
    WRAP_EGLD_ENDPOINT_NAME,
};
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc_scenario::{imports::TxTokenTransfer, rust_biguint, DebugApi};

pub mod generic_composable_tasks_test_setup;

#[test]
fn setup_test() {
    let _ = GenericCompTasksSetup::new(
        generic_composable_tasks::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
    );
}

#[test]
fn single_simple_transfer_test() {
    let mut setup = GenericCompTasksSetup::new(
        generic_composable_tasks::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
    );

    let transfer_amount = 100;
    let dest_address = setup.other_user_address.clone();

    // EGLD transfer
    setup
        .b_mock
        .execute_tx(
            &setup.user_address,
            &setup.tasks_wrapper,
            &rust_biguint!(transfer_amount),
            |sc| {
                let arg =
                    build_egld_simple_transfer_data::<DebugApi>(&dest_address, transfer_amount);

                let mut all_args = MultiValueEncoded::new();
                all_args.push(arg);

                sc.multi_call(all_args);
            },
        )
        .assert_ok();

    setup.b_mock.check_egld_balance(
        &setup.user_address,
        &rust_biguint!(USER_BALANCE - transfer_amount),
    );
    setup.b_mock.check_egld_balance(
        &dest_address,
        &rust_biguint!(USER_BALANCE + transfer_amount),
    );

    // ESDT transfer
    setup
        .b_mock
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.tasks_wrapper,
            RAND_ESDT_TOKEN_ID,
            0,
            &rust_biguint!(transfer_amount),
            |sc| {
                let arg = build_esdt_simple_transfer_data::<DebugApi>(
                    &dest_address,
                    vec![TxTokenTransfer {
                        token_identifier: RAND_ESDT_TOKEN_ID.to_vec(),
                        nonce: 0,
                        value: rust_biguint!(transfer_amount),
                    }],
                );

                let mut all_args = MultiValueEncoded::new();
                all_args.push(arg);

                sc.multi_call(all_args);
            },
        )
        .assert_ok();

    setup.b_mock.check_esdt_balance(
        &setup.user_address,
        RAND_ESDT_TOKEN_ID,
        &rust_biguint!(USER_BALANCE - transfer_amount),
    );
    setup.b_mock.check_esdt_balance(
        &dest_address,
        RAND_ESDT_TOKEN_ID,
        &rust_biguint!(transfer_amount),
    );
}

#[test]
fn double_simple_transfer_test() {
    let mut setup = GenericCompTasksSetup::new(
        generic_composable_tasks::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
    );

    let transfer_amount = 100;
    let dest_address = setup.other_user_address.clone();

    setup
        .b_mock
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.tasks_wrapper,
            RAND_ESDT_TOKEN_ID,
            0,
            &rust_biguint!(transfer_amount * 2),
            |sc| {
                let arg1 = build_esdt_simple_transfer_data::<DebugApi>(
                    &dest_address,
                    vec![TxTokenTransfer {
                        token_identifier: RAND_ESDT_TOKEN_ID.to_vec(),
                        nonce: 0,
                        value: rust_biguint!(transfer_amount),
                    }],
                );
                let arg2 = build_esdt_simple_transfer_data::<DebugApi>(
                    &dest_address,
                    vec![TxTokenTransfer {
                        token_identifier: RAND_ESDT_TOKEN_ID.to_vec(),
                        nonce: 0,
                        value: rust_biguint!(transfer_amount),
                    }],
                );

                let mut all_args = MultiValueEncoded::new();
                all_args.push(arg1);
                all_args.push(arg2);

                sc.multi_call(all_args);
            },
        )
        .assert_ok();

    setup.b_mock.check_esdt_balance(
        &setup.user_address,
        RAND_ESDT_TOKEN_ID,
        &rust_biguint!(USER_BALANCE - 2 * transfer_amount),
    );
    setup.b_mock.check_esdt_balance(
        &dest_address,
        RAND_ESDT_TOKEN_ID,
        &rust_biguint!(2 * transfer_amount),
    );
}

#[test]
fn sync_call_test() {
    let mut setup = GenericCompTasksSetup::new(
        generic_composable_tasks::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
    );

    let transfer_amount = 100;
    let dest_sc_address = setup.wegld_swap_wrapper.address_ref().clone();

    setup
        .b_mock
        .execute_tx(
            &setup.user_address,
            &setup.tasks_wrapper,
            &rust_biguint!(transfer_amount),
            |sc| {
                let arg = build_sync_call_egld_transfer_data::<DebugApi>(
                    &dest_sc_address,
                    transfer_amount,
                    WRAP_EGLD_ENDPOINT_NAME,
                    Vec::new(),
                );

                let mut all_args = MultiValueEncoded::new();
                all_args.push(arg);

                sc.multi_call(all_args);
            },
        )
        .assert_ok();

    setup.b_mock.check_egld_balance(
        &setup.user_address,
        &rust_biguint!(USER_BALANCE - transfer_amount),
    );
    setup
        .b_mock
        .check_egld_balance(&dest_sc_address, &rust_biguint!(transfer_amount));
    setup.b_mock.check_esdt_balance(
        &setup.user_address,
        WEGLD_TOKEN_ID,
        &rust_biguint!(transfer_amount),
    );

    setup
        .b_mock
        .execute_esdt_transfer(
            &setup.user_address,
            &setup.tasks_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(transfer_amount),
            |sc| {
                let arg = build_sync_call_esdt_transfer_data::<DebugApi>(
                    &dest_sc_address,
                    vec![TxTokenTransfer {
                        token_identifier: WEGLD_TOKEN_ID.to_vec(),
                        nonce: 0,
                        value: rust_biguint!(transfer_amount),
                    }],
                    UNWRAP_EGLD_ENDPOINT_NAME,
                    Vec::new(),
                );

                let mut all_args = MultiValueEncoded::new();
                all_args.push(arg);

                sc.multi_call(all_args);
            },
        )
        .assert_ok();

    setup
        .b_mock
        .check_egld_balance(&setup.user_address, &rust_biguint!(USER_BALANCE));
    setup
        .b_mock
        .check_egld_balance(&dest_sc_address, &rust_biguint!(0));
    setup
        .b_mock
        .check_esdt_balance(&setup.user_address, WEGLD_TOKEN_ID, &rust_biguint!(0));
}
