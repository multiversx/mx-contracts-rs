mod setup;

use dn404::dn404_proxy;
use multiversx_sc::{
    codec::IntoMultiValue,
    types::{EsdtTokenPayment, MultiValueEncoded, ReturnsResultUnmanaged, TestEsdtTransfer},
};
use multiversx_sc_scenario::{managed_biguint, rust_biguint, ScenarioTxRun};
use setup::{
    Dn404Setup, DN404_ADDRESS, FEE_COL, FEE_NONCE_2, FIRST_USER, FRACTAL_TOKEN_ID, NFT_TOKEN_ID,
    PRICE_COL, PRICE_NONCE_2, SECOND_USER, USER_BALANCE,
};

#[test]
fn setup_test() {
    Dn404Setup::new();
}

#[test]
fn user_buy_nfts_test() {
    let mut setup = Dn404Setup::new();

    let mut tokens_to_claim = MultiValueEncoded::new();
    for i in 5..=7 {
        let token =
            EsdtTokenPayment::new(NFT_TOKEN_ID.to_token_identifier(), i, managed_biguint!(1));
        tokens_to_claim.push(token.into_multi_value());
    }

    setup
        .b_mock
        .tx()
        .from(SECOND_USER)
        .to(DN404_ADDRESS)
        .typed(dn404_proxy::Dn404Proxy)
        .claim_basket_of_goods(tokens_to_claim)
        .with_esdt_transfer(TestEsdtTransfer(FRACTAL_TOKEN_ID, 0, 3 * PRICE_COL))
        .run();

    let basket_of_good = setup
        .b_mock
        .query()
        .to(DN404_ADDRESS)
        .typed(dn404_proxy::Dn404Proxy)
        .basket_of_goods()
        .returns(ReturnsResultUnmanaged)
        .run();

    assert_eq!(basket_of_good.len(), 1);

    let remaining_tokens = setup
        .b_mock
        .query()
        .to(DN404_ADDRESS)
        .typed(dn404_proxy::Dn404Proxy)
        .remaining_tokens(NFT_TOKEN_ID.to_token_identifier(), 5u64)
        .returns(ReturnsResultUnmanaged)
        .run();
    assert_eq!(remaining_tokens, rust_biguint!(0));

    setup
        .b_mock
        .check_account(SECOND_USER)
        .esdt_balance(FRACTAL_TOKEN_ID, (USER_BALANCE) - 3 * PRICE_COL);
}

#[test]
fn user_deposit_test() {
    let mut setup = Dn404Setup::new();
    let transfers = vec![
        TestEsdtTransfer(NFT_TOKEN_ID, 1, 1),
        TestEsdtTransfer(NFT_TOKEN_ID, 2, 1),
        TestEsdtTransfer(NFT_TOKEN_ID, 3, 1),
        TestEsdtTransfer(NFT_TOKEN_ID, 4, 1),
    ];

    setup
        .b_mock
        .tx()
        .from(FIRST_USER)
        .to(DN404_ADDRESS)
        .typed(dn404_proxy::Dn404Proxy)
        .deposit_basket_of_goods()
        .multi_esdt(transfers)
        .run();

    let expected_user_balance = PRICE_NONCE_2 - FEE_NONCE_2 + 3 * PRICE_COL - 3 * FEE_COL;
    setup
        .b_mock
        .check_account(FIRST_USER)
        .esdt_balance(FRACTAL_TOKEN_ID, expected_user_balance);
}
