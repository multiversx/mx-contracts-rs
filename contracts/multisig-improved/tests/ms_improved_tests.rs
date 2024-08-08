pub mod ms_improved_setup;

use adder::Adder;
use can_execute_mock::CanExecuteMock;
use ms_improved_setup::*;
use multisig_improved::{
    common_types::{
        action::{Action, CallActionData},
        signature::{ActionType, SignatureArg, SignatureType},
        user_role::UserRole,
    },
    external::views::ViewsModule,
    ms_endpoints::{
        discard::DiscardEndpointsModule, perform::PerformEndpointsModule,
        propose::ProposeEndpointsModule, sign::SignEndpointsModule,
    },
    Multisig,
};
use multiversx_sc::{
    codec::TopEncode,
    imports::OptionalValue,
    types::{FunctionCall, ManagedArgBuffer, ManagedBuffer, ManagedVec, MultiValueEncoded},
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
fn change_quorum_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    // try change quorum > board size
    let new_quorum = 3;
    let action_id = ms_setup.propose_change_quorum(new_quorum);
    ms_setup.sign(action_id, 0);
    ms_setup.perform_and_expect_err(action_id, "quorum cannot exceed board size");

    // try discard before unsigning
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.discard_action_endpoint(action_id);
            },
        )
        .assert_user_error("cannot discard action with valid signatures");

    // unsign and discard action
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.unsign(action_id);
            },
        )
        .assert_ok();

    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.second_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.unsign(action_id);
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
                sc.discard_action_endpoint(action_id);
            },
        )
        .assert_ok();

    // try sign discarded action
    let signer_addr = ms_setup.first_board_member.clone();
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut signatures = MultiValueEncoded::new();
                signatures.push(SignatureArg {
                    user_address: managed_address!(&signer_addr),
                    nonce: 1,
                    action_type: ActionType::SimpleAction,
                    raw_sig_bytes: managed_buffer!(b"signature"),
                    signature_type: SignatureType::Ed25519, // unused
                });

                sc.sign(action_id, signatures);
            },
        )
        .assert_user_error("action does not exist");

    // add another board member
    let new_board_member = ms_setup.b_mock.create_user_account(&rust_biguint!(0));
    ms_setup.expect_user_role(&new_board_member, UserRole::None);

    let action_id = ms_setup.propose_add_board_member(&new_board_member);
    ms_setup.sign(action_id, 1);
    ms_setup.perform(action_id);
    ms_setup.expect_user_role(&new_board_member, UserRole::BoardMember);

    // change quorum to 3
    let action_id = ms_setup.propose_change_quorum(new_quorum);
    ms_setup.sign(action_id, 2);
    ms_setup.perform(action_id);
}

#[test]
fn transfer_execute_to_user_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let new_user = ms_setup.b_mock.create_user_account(&rust_biguint!(0));
    let egld_balance = 100;
    ms_setup
        .b_mock
        .set_egld_balance(&ms_setup.first_board_member, &rust_biguint!(egld_balance));

    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(egld_balance),
            |sc| {
                sc.deposit();
            },
        )
        .assert_ok();

    ms_setup.b_mock.check_egld_balance(
        ms_setup.ms_wrapper.address_ref(),
        &rust_biguint!(egld_balance),
    );

    // failed attempt
    let mut action_id = 0;
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                action_id = sc
                    .propose_transfer_execute(
                        managed_address!(&new_user),
                        managed_biguint!(0),
                        None,
                        FunctionCall::empty(),
                        OptionalValue::None,
                    )
                    .into_option()
                    .unwrap();
            },
        )
        .assert_user_error("proposed action has no effect");

    // propose
    let action_id = ms_setup.propose_transfer_execute(&new_user, egld_balance, &[], Vec::new());
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);

    ms_setup
        .b_mock
        .check_egld_balance(ms_setup.ms_wrapper.address_ref(), &rust_biguint!(0));
    ms_setup
        .b_mock
        .check_egld_balance(&new_user, &rust_biguint!(egld_balance));
}

#[test]
fn transfer_execute_sc_call_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let args = [&[5u8][..]].to_vec();
    let action_id = ms_setup.propose_transfer_execute(
        &ms_setup.adder_wrapper.address_ref().clone(),
        0,
        b"add",
        args,
    );
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);

    ms_setup
        .b_mock
        .execute_query(&ms_setup.adder_wrapper, |sc| {
            assert_eq!(sc.sum().get(), 5);
        })
        .assert_ok();
}

#[test]
fn transfer_execute_batch_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let adder_addr = ms_setup.adder_wrapper.address_ref().clone();
    let mut group_id = 0;
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.first_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = ManagedVec::new();
                let mut encoded_arg = ManagedBuffer::<DebugApi>::new();
                let _ = 5u32.top_encode(&mut encoded_arg);
                args.push(encoded_arg);

                let single_action = Action::SendTransferExecuteEgld(CallActionData {
                    to: managed_address!(&adder_addr),
                    egld_amount: managed_biguint!(0),
                    opt_gas_limit: None,
                    endpoint_name: managed_buffer!(b"add"),
                    arguments: args,
                });

                let mut multi_action_vec = MultiValueEncoded::new();
                multi_action_vec.push(single_action.clone());
                multi_action_vec.push(single_action);

                group_id = sc.propose_batch(multi_action_vec);
            },
        )
        .assert_ok();

    let signer_addr = ms_setup.second_board_member.clone();
    ms_setup
        .b_mock
        .execute_tx(
            &ms_setup.second_board_member,
            &ms_setup.ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut signatures = MultiValueEncoded::new();
                signatures.push(SignatureArg {
                    user_address: managed_address!(&signer_addr),
                    nonce: 0,
                    action_type: ActionType::Group,
                    raw_sig_bytes: managed_buffer!(b"signature"),
                    signature_type: SignatureType::Ed25519, // unused
                });

                sc.sign_batch_and_perform(group_id, signatures)
            },
        )
        .assert_ok();

    ms_setup
        .b_mock
        .execute_query(&ms_setup.adder_wrapper, |sc| {
            assert_eq!(sc.sum().get(), 10);
        })
        .assert_ok();
}

#[test]
fn async_call_to_sc_test() {
    let mut ms_setup = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);

    let args = [&[5u8][..]].to_vec();
    let action_id = ms_setup.propose_async_call(
        &ms_setup.adder_wrapper.address_ref().clone(),
        0,
        b"add",
        args,
    );
    ms_setup.sign(action_id, 0);
    ms_setup.perform(action_id);

    ms_setup
        .b_mock
        .execute_query(&ms_setup.adder_wrapper, |sc| {
            assert_eq!(sc.sum().get(), 5);
        })
        .assert_ok();
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
