#![allow(deprecated)]

use fair_launch::{
    common::Percentage, exchange_actions::ExchangeActionsModule, token_info::TokenInfoModule,
    FairLaunch,
};
use multiversx_sc::{
    storage::mappers::StorageTokenWrapper,
    types::{Address, MultiValueEncoded},
};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use pair_mock::PairMock;

pub static TOKEN_ID: &[u8] = b"MYTOKEN-123456";
pub static OTHER_TOKEN_ID: &[u8] = b"OTHER-123456";
pub const LAUNCH_DURATION_BLOCKS: u64 = 100;
pub const ACCOUNT_BUY_LIMIT: u64 = 2_000;
pub const TX_BUY_LIMIT: u64 = 1_000;
pub const BUY_FEE_PERCENTAGE_START: Percentage = 9_000;
pub const BUY_FEE_PERCENTAGE_END: Percentage = 1_000;
pub const SELL_FEE_PERCENTAGE_START: Percentage = 10_000;
pub const SELL_FEE_PERCENTAGE_END: Percentage = 5_000;

pub struct FairLaunchSetup<FairLaunchObjBuilder, PairObjBuilder>
where
    FairLaunchObjBuilder: 'static + Copy + Fn() -> fair_launch::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair_mock::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub first_user_address: Address,
    pub second_user_address: Address,
    pub fl_wrapper: ContractObjWrapper<fair_launch::ContractObj<DebugApi>, FairLaunchObjBuilder>,
    pub pair_wrapper: ContractObjWrapper<pair_mock::ContractObj<DebugApi>, PairObjBuilder>,
}

impl<FairLaunchObjBuilder, PairObjBuilder> FairLaunchSetup<FairLaunchObjBuilder, PairObjBuilder>
where
    FairLaunchObjBuilder: 'static + Copy + Fn() -> fair_launch::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair_mock::ContractObj<DebugApi>,
{
    pub fn new(fl_builder: FairLaunchObjBuilder, pair_builder: PairObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let first_user_address = b_mock.create_user_account(&rust_zero);
        let second_user_address = b_mock.create_user_account(&rust_zero);
        let owner_address = b_mock.create_user_account(&rust_zero);

        let pair_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            pair_builder,
            "pair wasm path",
        );

        b_mock
            .execute_tx(&owner_address, &pair_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_token_id!(TOKEN_ID),
                    managed_token_id!(OTHER_TOKEN_ID),
                );
            })
            .assert_ok();

        let fl_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            fl_builder,
            "fair launch wasm path",
        );
        b_mock
            .execute_tx(&owner_address, &fl_wrapper, &rust_zero, |sc| {
                sc.init(
                    LAUNCH_DURATION_BLOCKS,
                    managed_biguint!(ACCOUNT_BUY_LIMIT),
                    managed_biguint!(TX_BUY_LIMIT),
                    BUY_FEE_PERCENTAGE_START,
                    BUY_FEE_PERCENTAGE_END,
                    SELL_FEE_PERCENTAGE_START,
                    SELL_FEE_PERCENTAGE_END,
                );
                sc.non_fungible_token()
                    .set_token_id(managed_token_id!(TOKEN_ID));

                let mut pairs = MultiValueEncoded::new();
                pairs.push(
                    (
                        managed_buffer!(b"swapTokensFixedInput"),
                        4_000u32,
                        false,
                        0u32,
                        false,
                    )
                        .into(),
                );
                sc.add_exchange_endpoint(managed_address!(pair_wrapper.address_ref()), pairs);
                sc.set_paused(false);
            })
            .assert_ok();

        Self {
            b_mock,
            owner_address,
            first_user_address,
            second_user_address,
            fl_wrapper,
            pair_wrapper,
        }
    }
}
