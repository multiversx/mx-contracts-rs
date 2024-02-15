use dn404::{
    available_tokens::AvailableTokensModule, fee::FeeModule, price::PriceModule, Dn404, Percentage,
};
use multiversx_sc::{
    codec::Empty,
    types::{Address, EsdtLocalRole, MultiValueEncoded},
};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper, TxTokenTransfer},
    DebugApi,
};

pub static FRACTAL_TOKEN_ID: &[u8] = b"FRACTAL-123456";
pub static NFT_TOKEN_ID: &[u8] = b"NFT-123456";
pub const PRICE_NONCE_2: u64 = 50;
pub const PRICE_COL: u64 = 25;
pub const FEE_NONCE_2: u64 = 10;
pub const FEE_COL: u64 = 5;
pub const FEE_BASKET: Percentage = 1_000; // 10%
pub const USER_BALANCE: u64 = 1_000;

pub struct Dn404Setup<Dn404Builder>
where
    Dn404Builder: 'static + Copy + Fn() -> dn404::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_addr: Address,
    pub first_user_addr: Address,
    pub second_user_addr: Address,
    pub dn404_wrapper: ContractObjWrapper<dn404::ContractObj<DebugApi>, Dn404Builder>,
}

impl<Dn404Builder> Dn404Setup<Dn404Builder>
where
    Dn404Builder: 'static + Copy + Fn() -> dn404::ContractObj<DebugApi>,
{
    pub fn new(builder: Dn404Builder) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_addr = b_mock.create_user_account(&rust_zero);
        let first_user_addr = b_mock.create_user_account(&rust_zero);
        let second_user_addr = b_mock.create_user_account(&rust_zero);

        let dn404_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner_addr), builder, "dn404 wasm path");
        b_mock
            .execute_tx(&owner_addr, &dn404_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_token_id!(FRACTAL_TOKEN_ID),
                    MultiValueEncoded::new(),
                );
                sc.set_internal_price_for_token(
                    managed_token_id!(NFT_TOKEN_ID),
                    2,
                    managed_biguint!(PRICE_NONCE_2),
                );
                sc.set_internal_price_for_collection(
                    managed_token_id!(NFT_TOKEN_ID),
                    managed_biguint!(PRICE_COL),
                );
                sc.set_fee_for_fractionalizing_nft(
                    managed_token_id!(NFT_TOKEN_ID),
                    2,
                    managed_biguint!(FEE_NONCE_2),
                );
                sc.set_fee_for_fractionalizing_collection(
                    managed_token_id!(NFT_TOKEN_ID),
                    managed_biguint!(FEE_COL),
                );
                sc.set_fee_for_deposit_basket_of_goods(FEE_BASKET);

                sc.set_paused(false);
            })
            .assert_ok();

        b_mock.set_nft_balance(&owner_addr, NFT_TOKEN_ID, 5, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&owner_addr, NFT_TOKEN_ID, 6, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&owner_addr, NFT_TOKEN_ID, 7, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&owner_addr, NFT_TOKEN_ID, 8, &rust_biguint!(1), &Empty);
        let transfers = [
            TxTokenTransfer {
                token_identifier: NFT_TOKEN_ID.to_vec(),
                nonce: 5,
                value: rust_biguint!(1),
            },
            TxTokenTransfer {
                token_identifier: NFT_TOKEN_ID.to_vec(),
                nonce: 6,
                value: rust_biguint!(1),
            },
            TxTokenTransfer {
                token_identifier: NFT_TOKEN_ID.to_vec(),
                nonce: 7,
                value: rust_biguint!(1),
            },
            TxTokenTransfer {
                token_identifier: NFT_TOKEN_ID.to_vec(),
                nonce: 8,
                value: rust_biguint!(1),
            },
        ];

        b_mock
            .execute_esdt_multi_transfer(&owner_addr, &dn404_wrapper, &transfers, |sc| {
                sc.deposit();
            })
            .assert_ok();

        b_mock.set_nft_balance(&first_user_addr, NFT_TOKEN_ID, 1, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&first_user_addr, NFT_TOKEN_ID, 2, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&first_user_addr, NFT_TOKEN_ID, 3, &rust_biguint!(1), &Empty);
        b_mock.set_nft_balance(&first_user_addr, NFT_TOKEN_ID, 4, &rust_biguint!(1), &Empty);
        b_mock.set_esdt_balance(
            &second_user_addr,
            FRACTAL_TOKEN_ID,
            &rust_biguint!(USER_BALANCE),
        );

        b_mock.set_esdt_local_roles(
            dn404_wrapper.address_ref(),
            FRACTAL_TOKEN_ID,
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        Dn404Setup {
            b_mock,
            owner_addr,
            first_user_addr,
            second_user_addr,
            dn404_wrapper,
        }
    }
}
