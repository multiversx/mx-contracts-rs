pub mod ms_improved_setup;

use adder::Adder;
use can_execute_mock::CanExecuteMock;
use ms_improved_setup::*;
use multisig_improved::{
    common_types::{
        signature::{ActionType, SignatureArg, SignatureType},
        user_role::UserRole,
    },
    external::views::ViewsModule,
    ms_endpoints::{
        perform::PerformEndpointsModule, propose::ProposeEndpointsModule, sign::SignEndpointsModule,
    },
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{FunctionCall, ManagedArgBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, rust_biguint, DebugApi,
};

#[test]
fn init_test() {
    let _ = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);
}

#[test]
fn add_board_member_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let new_board_member = ms_setup.b_mock.create_user_account(&rust_biguint!(0));
    ms_setup.expect_user_role(&new_board_member, UserRole::None);

    let action_id = ms_setup.propose_add_board_member(&new_board_member);
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);
    ms_setup.expect_user_role(&new_board_member, UserRole::BoardMember);

    let first_board_member = ms_setup.first_board_member.clone();
    let second_board_member = ms_setup.second_board_member.clone();
    ms_setup
        .b_mock
        .execute_query(&ms_setup.ms_wrapper, |sc| {
            let mut expected_board_members = MultiValueEncoded::new();
            expected_board_members.push(managed_address!(&first_board_member));
            expected_board_members.push(managed_address!(&second_board_member));
            expected_board_members.push(managed_address!(&new_board_member));

            let actual_board_members = sc.get_all_board_members();
            assert_eq!(expected_board_members, actual_board_members);
        })
        .assert_ok();
}

#[test]
fn add_proposer_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let new_proposer = ms_setup.b_mock.create_user_account(&rust_biguint!(0));
    ms_setup.expect_user_role(&new_proposer, UserRole::None);

    let action_id = ms_setup.propose_add_proposer(&new_proposer);
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);
    ms_setup.expect_user_role(&new_proposer, UserRole::Proposer);

    ms_setup
        .b_mock
        .execute_query(&ms_setup.ms_wrapper, |sc| {
            let mut expected_proposers = MultiValueEncoded::new();
            expected_proposers.push(managed_address!(&new_proposer));

            let actual_proposers = sc.get_all_proposers();
            assert_eq!(expected_proposers, actual_proposers);
        })
        .assert_ok();
}

#[test]
fn remove_proposer_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let new_proposer = ms_setup.b_mock.create_user_account(&rust_biguint!(0));
    ms_setup.expect_user_role(&new_proposer, UserRole::None);

    let action_id = ms_setup.propose_add_proposer(&new_proposer);
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);
    ms_setup.expect_user_role(&new_proposer, UserRole::Proposer);

    ms_setup
        .b_mock
        .execute_query(&ms_setup.ms_wrapper, |sc| {
            let mut expected_proposers = MultiValueEncoded::new();
            expected_proposers.push(managed_address!(&new_proposer));

            let actual_proposers = sc.get_all_proposers();
            assert_eq!(expected_proposers, actual_proposers);
        })
        .assert_ok();

    let action_id = ms_setup.propose_remove_user(&new_proposer);
    ms_setup.sign(action_id, 1);
    ms_setup.perform(action_id);

    ms_setup.expect_user_role(&new_proposer, UserRole::None);
}

#[test]
fn try_remove_all_board_members_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let action_id = ms_setup.propose_remove_user(&ms_setup.first_board_member.clone());
    ms_setup.sign(action_id, 0);
    ms_setup.perform_and_expect_err(action_id, "quorum cannot exceed board size");
}

#[test]
fn add_can_execute_module_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);
    let can_execute_mock = ms_setup.b_mock.create_sc_account(
        &rust_biguint!(0),
        Some(&ms_setup.ms_owner),
        CanExecuteMock::new,
        "canExecute mock",
    );

    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.propose_add_module(
                    managed_address!(can_execute_mock.address_ref()),
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // try execute action without enough signatures
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let _ = sc.perform_action_endpoint(1);
            },
        )
        .assert_user_error("quorum has not been reached");

    // other user sign
    let other_board_member = ms_setup.second_board_member.clone();
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut signatures = MultiValueEncoded::new();
                signatures.push(SignatureArg {
                    user_address: managed_address!(&other_board_member),
                    nonce: 0,
                    action_type: ActionType::SimpleAction,
                    signature_type: SignatureType::Ed25519, // unused
                    raw_sig_bytes: managed_buffer!(b"signature"),
                });

                sc.sign(1, signatures)
            },
        )
        .assert_ok();

    // execute action ok
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let _ = sc.perform_action_endpoint(1);
            },
        )
        .assert_ok();

    // execute action via canExecute -> no signatures required
    let adder_addr = ms_setup.adder_wrapper.address_ref();
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut add_function_args = ManagedArgBuffer::new();
                add_function_args
                    .push_arg::<multiversx_sc::types::BigUint<DebugApi>>(managed_biguint!(5));

                let func_result = sc.propose_transfer_execute(
                    managed_address!(adder_addr),
                    managed_biguint!(0),
                    Option::None,
                    FunctionCall {
                        function_name: managed_buffer!(b"add"),
                        arg_buffer: add_function_args,
                    },
                    OptionalValue::None,
                );
                assert!(func_result.is_none());
            },
        )
        .assert_ok();

    // check action was actually executed inside adder
    ms_setup
        .b_mock
        .execute_query(&ms_setup.adder_wrapper, |sc| {
            assert_eq!(sc.sum().get(), managed_biguint!(5));
        })
        .assert_ok();

    // remove module
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.propose_remove_module(
                    managed_address!(can_execute_mock.address_ref()),
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // ID is 3, as even though previous proposal didn't actually register, an action ID is still used
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut signatures = MultiValueEncoded::new();
                signatures.push(SignatureArg {
                    user_address: managed_address!(&other_board_member),
                    nonce: 1,
                    action_type: ActionType::SimpleAction,
                    signature_type: SignatureType::Ed25519, // unused
                    raw_sig_bytes: managed_buffer!(b"signature"),
                });

                sc.sign(3, signatures)
            },
        )
        .assert_ok();

    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let _ = sc.perform_action_endpoint(3);
            },
        )
        .assert_ok();

    // user try execute action directly again
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut add_function_args = ManagedArgBuffer::new();
                add_function_args
                    .push_arg::<multiversx_sc::types::BigUint<DebugApi>>(managed_biguint!(5));

                let func_result = sc.propose_transfer_execute(
                    managed_address!(adder_addr),
                    managed_biguint!(0),
                    Option::None,
                    FunctionCall {
                        function_name: managed_buffer!(b"add"),
                        arg_buffer: add_function_args,
                    },
                    OptionalValue::None,
                );

                // action didn't execute, it returned action_id
                assert!(func_result.is_some());
            },
        )
        .assert_ok();
}
