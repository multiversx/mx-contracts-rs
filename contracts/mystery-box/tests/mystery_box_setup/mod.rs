use multiversx_sc::types::{Address, EgldOrEsdtTokenIdentifier, EsdtLocalRole, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_biguint, managed_buffer, managed_token_id, rust_biguint, whitebox::*, DebugApi,
};
use mystery_box::{
    config::{CooldownType, RewardType},
    MysteryBox,
};

pub const MYSTERY_BOX_WASM_PATH: &str = "mystery-box/output/mystery-box.wasm";
pub const MB_TOKEN_ID: &[u8] = b"MBTOK-abcdef";

#[allow(dead_code)]
pub struct MysteryBoxSetup<MysteryBoxObjBuilder>
where
    MysteryBoxObjBuilder: 'static + Copy + Fn() -> mystery_box::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub mystery_box_wrapper:
        ContractObjWrapper<mystery_box::ContractObj<DebugApi>, MysteryBoxObjBuilder>,
}

impl<MysteryBoxObjBuilder> MysteryBoxSetup<MysteryBoxObjBuilder>
where
    MysteryBoxObjBuilder: 'static + Copy + Fn() -> mystery_box::ContractObj<DebugApi>,
{
    pub fn new(mystery_box_builder: MysteryBoxObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));
        let mystery_box_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_addr),
            mystery_box_builder,
            MYSTERY_BOX_WASM_PATH,
        );

        b_mock
            .execute_tx(&owner_addr, &mystery_box_wrapper, &rust_zero, |sc| {
                sc.init(managed_token_id!(MB_TOKEN_ID));
            })
            .assert_ok();

        let mb_token_roles = [
            EsdtLocalRole::NftCreate,
            EsdtLocalRole::NftAddQuantity,
            EsdtLocalRole::NftBurn,
        ];
        b_mock.set_esdt_local_roles(
            mystery_box_wrapper.address_ref(),
            MB_TOKEN_ID,
            &mb_token_roles[..],
        );

        Self::internal_setup_mystery_box(&owner_addr, &mystery_box_wrapper, &mut b_mock);

        MysteryBoxSetup {
            b_mock,
            owner_address: owner_addr,
            mystery_box_wrapper,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn internal_setup_mystery_box(
        owner_address: &Address,
        mb_wrapper: &ContractObjWrapper<mystery_box::ContractObj<DebugApi>, MysteryBoxObjBuilder>,
        b_mock: &mut BlockchainStateWrapper,
    ) where
        MysteryBoxObjBuilder: 'static + Copy + Fn() -> mystery_box::ContractObj<DebugApi>,
    {
        b_mock
            .execute_tx(owner_address, mb_wrapper, &rust_biguint!(0), |sc| {
                let mut rewards_list = MultiValueEncoded::new();
                let mut reward = (
                    RewardType::ExperiencePoints,
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(10_000),
                    managed_buffer!(b"ExperiencePoints"),
                    3_000u64,
                    CooldownType::None,
                    0,
                    0,
                )
                    .into();
                rewards_list.push(reward);

                reward = (
                    RewardType::MysteryBox,
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(MB_TOKEN_ID)),
                    managed_biguint!(1),
                    managed_buffer!(b"MysteryBox"),
                    1_500u64,
                    CooldownType::None,
                    0,
                    0,
                )
                    .into();
                rewards_list.push(reward);

                reward = (
                    RewardType::PercentValue,
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(1_500u64),
                    managed_buffer!(b"Percent"),
                    1_000u64,
                    CooldownType::None,
                    0,
                    0,
                )
                    .into();
                rewards_list.push(reward);

                reward = (
                    RewardType::FixedValue,
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(10u64),
                    managed_buffer!(b"FixedValue"),
                    4_400u64,
                    CooldownType::ResetOnCooldown,
                    2,
                    2,
                )
                    .into();
                rewards_list.push(reward);

                reward = (
                    RewardType::CustomReward,
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(400u64),
                    managed_buffer!(b"CustomText"),
                    100u64,
                    CooldownType::Lifetime,
                    1,
                    0,
                )
                    .into();
                rewards_list.push(reward);

                sc.setup_mystery_box(rewards_list);

                let mut uris = MultiValueEncoded::new();
                uris.push(managed_buffer!(b"www.cool_nft.com/my_nft.jpg"));
                uris.push(managed_buffer!(b"www.cool_nft.com/my_nft.json"));
                sc.update_mystery_box_uris(uris);
            })
            .assert_ok();
    }

    pub fn create_mystery_box(&mut self, amount: u64) -> u64 {
        let mut new_nonce = 0;
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.mystery_box_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let payment = sc.create_mystery_box(managed_biguint!(amount));
                    new_nonce = payment.token_nonce;

                    assert_eq!(payment.amount, managed_biguint!(amount));
                },
            )
            .assert_ok();

        new_nonce
    }

    pub fn open_mystery_box(&mut self, mb_token_nonce: u64) {
        self.b_mock
            .execute_esdt_transfer(
                &self.owner_address,
                &self.mystery_box_wrapper,
                MB_TOKEN_ID,
                mb_token_nonce,
                &rust_biguint!(1),
                |sc| {
                    sc.open_mystery_box();
                },
            )
            .assert_ok();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn setup_mystery_box(&mut self) {
        Self::internal_setup_mystery_box(
            &self.owner_address,
            &self.mystery_box_wrapper,
            &mut self.b_mock,
        );
    }
}
