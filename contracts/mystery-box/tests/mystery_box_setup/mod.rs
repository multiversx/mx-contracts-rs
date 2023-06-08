use multiversx_sc::types::{Address, EgldOrEsdtTokenIdentifier, EsdtLocalRole, ManagedVec};
use multiversx_sc_scenario::{
    managed_biguint, managed_buffer, managed_token_id, rust_biguint, whitebox::*, DebugApi,
};
use mystery_box::{
    config::{Reward, RewardType},
    MysteryBox,
};

pub const MYSTERY_BOX_WASM_PATH: &str = "mystery-box/output/mystery-box.wasm";
pub const MB_TOKEN_ID: &[u8] = b"MBTOK-abcdef";
pub const MYSTERY_BOX_COOLDOWN_PERIOD: u64 = 1;

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
                sc.init(managed_token_id!(MB_TOKEN_ID), MYSTERY_BOX_COOLDOWN_PERIOD);
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

        Self::setup_mystery_box(
            900,
            5_999,
            1,
            3_000,
            0,
            1_500,
            1_000,
            0,
            50,
            1,
            1,
            &owner_addr,
            &mystery_box_wrapper,
            &mut b_mock,
        );

        MysteryBoxSetup {
            b_mock,
            owner_address: owner_addr,
            mystery_box_wrapper,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn setup_mystery_box(
        experience_points_amount: u64,
        experience_points_percentage: u64,
        experience_points_cooldown: u64,
        sft_reward_percentage: u64,
        sft_reward_cooldown: u64,
        percent_reward_amount: u64,
        percent_reward_percentage: u64,
        percent_reward_cooldown: u64,
        fixed_value_reward_amount: u64,
        fixed_value_reward_percentage: u64,
        fixed_value_reward_cooldown: u64,
        owner_address: &Address,
        mb_wrapper: &ContractObjWrapper<mystery_box::ContractObj<DebugApi>, MysteryBoxObjBuilder>,
        b_mock: &mut BlockchainStateWrapper,
    ) where
        MysteryBoxObjBuilder: 'static + Copy + Fn() -> mystery_box::ContractObj<DebugApi>,
    {
        b_mock
            .execute_tx(owner_address, mb_wrapper, &rust_biguint!(0), |sc| {
                let mut rewards_list = ManagedVec::new();
                let mut reward = Reward {
                    reward_type: RewardType::ExperiencePoints,
                    reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                    value: managed_biguint!(experience_points_amount),
                    percentage_chance: experience_points_percentage,
                    epochs_cooldown: experience_points_cooldown,
                };
                rewards_list.push(reward);

                reward = Reward {
                    reward_type: RewardType::MysteryBox,
                    reward_token_id: EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(
                        MB_TOKEN_ID
                    )),
                    value: managed_biguint!(1),
                    percentage_chance: sft_reward_percentage,
                    epochs_cooldown: sft_reward_cooldown,
                };
                rewards_list.push(reward);

                reward = Reward {
                    reward_type: RewardType::Percent,
                    reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                    value: managed_biguint!(percent_reward_amount),
                    percentage_chance: percent_reward_percentage,
                    epochs_cooldown: percent_reward_cooldown,
                };
                rewards_list.push(reward);

                reward = Reward {
                    reward_type: RewardType::FixedValue,
                    reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                    value: managed_biguint!(fixed_value_reward_amount),
                    percentage_chance: fixed_value_reward_percentage,
                    epochs_cooldown: fixed_value_reward_cooldown,
                };
                rewards_list.push(reward);
                sc.setup_mystery_box(rewards_list);

                let mut uris = ManagedVec::new();
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

    pub fn open_mystery_box_cooldown_error_expected(&mut self, mb_token_nonce: u64) {
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
            .assert_error(4, "Mystery box cannot be opened yet");
    }
}
