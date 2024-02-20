#![allow(deprecated)]

pub mod dn404_setup;

use dn404::available_tokens::AvailableTokensModule;
use dn404_setup::*;
use multiversx_sc::{
    codec::{Empty, IntoMultiValue},
    types::{EsdtTokenPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_biguint, managed_token_id, rust_biguint, testing_framework::TxTokenTransfer,
};

#[test]
fn setup_test() {
    let _ = Dn404Setup::new(dn404::contract_obj);
}

#[test]
fn user_buy_nfts_test() {
    let mut setup = Dn404Setup::new(dn404::contract_obj);
    setup
        .b_mock
        .execute_esdt_transfer(
            &setup.second_user_addr,
            &setup.dn404_wrapper,
            FRACTAL_TOKEN_ID,
            0,
            &rust_biguint!(3 * PRICE_COL),
            |sc| {
                let mut tokens_to_claim = MultiValueEncoded::new();
                for i in 5..=7 {
                    let token = EsdtTokenPayment::new(
                        managed_token_id!(NFT_TOKEN_ID),
                        i,
                        managed_biguint!(1),
                    );
                    tokens_to_claim.push(token.into_multi_value());
                }

                sc.claim_basket_of_goods(tokens_to_claim);
                assert_eq!(sc.basket_of_goods().len(), 1);
                assert_eq!(
                    sc.remaining_tokens(&managed_token_id!(NFT_TOKEN_ID), 5)
                        .get(),
                    0
                );
            },
        )
        .assert_ok();

    setup.b_mock.check_esdt_balance(
        &setup.second_user_addr,
        FRACTAL_TOKEN_ID,
        &(rust_biguint!(USER_BALANCE) - 3 * PRICE_COL),
    );
    setup.b_mock.check_nft_balance::<Empty>(
        &setup.second_user_addr,
        NFT_TOKEN_ID,
        5,
        &rust_biguint!(1),
        None,
    );
}

#[test]
fn user_deposit_test() {
    let mut setup = Dn404Setup::new(dn404::contract_obj);
    let transfers = [
        TxTokenTransfer {
            token_identifier: NFT_TOKEN_ID.to_vec(),
            nonce: 1,
            value: rust_biguint!(1),
        },
        TxTokenTransfer {
            token_identifier: NFT_TOKEN_ID.to_vec(),
            nonce: 2,
            value: rust_biguint!(1),
        },
        TxTokenTransfer {
            token_identifier: NFT_TOKEN_ID.to_vec(),
            nonce: 3,
            value: rust_biguint!(1),
        },
        TxTokenTransfer {
            token_identifier: NFT_TOKEN_ID.to_vec(),
            nonce: 4,
            value: rust_biguint!(1),
        },
    ];

    setup
        .b_mock
        .execute_esdt_multi_transfer(
            &setup.first_user_addr,
            &setup.dn404_wrapper,
            &transfers,
            |sc| {
                sc.deposit_basket_of_goods();
            },
        )
        .assert_ok();

    let expected_user_balance = PRICE_NONCE_2 - FEE_NONCE_2 + 3 * PRICE_COL - 3 * FEE_COL;
    setup.b_mock.check_esdt_balance(
        &setup.first_user_addr,
        FRACTAL_TOKEN_ID,
        &rust_biguint!(expected_user_balance),
    )
}
