use multiversx_sc::types::{Address, EgldOrEsdtTokenIdentifier, EsdtLocalRole, ManagedVec};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, whitebox::*, DebugApi,
};
use mystery_box::{
    config::{Reward, RewardType},
    MysteryBox,
};

pub const MYSTERY_BOX_WASM_PATH: &str = "mystery-box/output/mystery-box.wasm";
pub const MB_TOKEN_ID: &[u8] = b"MBTOK-abcdef";
pub const SFT_REWARD_TOKEN_ID: &[u8] = b"SFTR-abcdef";
pub const PRICE_DECIMALS: u64 = 1_000_000_000_000_000_000;

#[allow(dead_code)]
pub struct MysteryBoxSetup<MysteryBoxObjBuilder>
where
    MysteryBoxObjBuilder: 'static + Copy + Fn() -> mystery_box::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
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
        let owner_addr = b_mock.create_user_account(&rust_zero);
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

        let user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));

        MysteryBoxSetup {
            b_mock,
            owner_address: owner_addr,
            user_address: user_addr,
            mystery_box_wrapper,
        }
    }

    pub fn create_mystery_box(
        &mut self,
        experience_points_amount: u64,
        experience_points_percentage: u64,
        nft_reward_nonce: u64,
        nft_reward_percentage: u64,
        percent_reward_amount: u64,
        percent_reward_percentage: u64,
        fixed_value_reward_amount: u64,
        fixed_value_reward_percentage: u64,
        mb_token_expected_nonce: u64,
        mb_token_expected_amount: u64,
    ) -> u64 {
        let mut mb_token_nonce = 0;
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.mystery_box_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut rewards_list = ManagedVec::new();
                    let mut reward = Reward {
                        reward_type: RewardType::ExperiencePoints,
                        reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                        reward_token_nonce: 0,
                        value: managed_biguint!(experience_points_amount),
                        percentage_chance: experience_points_percentage,
                        epochs_cooldown: 0u64,
                    };
                    rewards_list.push(reward);

                    reward = Reward {
                        reward_type: RewardType::NFT,
                        reward_token_id: EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(
                            SFT_REWARD_TOKEN_ID
                        )),
                        reward_token_nonce: nft_reward_nonce,
                        value: managed_biguint!(1),
                        percentage_chance: nft_reward_percentage,
                        epochs_cooldown: 0u64,
                    };
                    rewards_list.push(reward);

                    reward = Reward {
                        reward_type: RewardType::Percent,
                        reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                        reward_token_nonce: 0,
                        value: managed_biguint!(percent_reward_amount),
                        percentage_chance: percent_reward_percentage,
                        epochs_cooldown: 0u64,
                    };
                    rewards_list.push(reward);

                    reward = Reward {
                        reward_type: RewardType::FixedValue,
                        reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
                        reward_token_nonce: 0,
                        value: managed_biguint!(fixed_value_reward_amount) * PRICE_DECIMALS,
                        percentage_chance: fixed_value_reward_percentage,
                        epochs_cooldown: 1u64,
                    };
                    rewards_list.push(reward);
                    let output_payment =
                        sc.create_mystery_box(managed_address!(&self.user_address), rewards_list);

                    assert_eq!(
                        output_payment.token_identifier,
                        managed_token_id!(MB_TOKEN_ID)
                    );
                    assert_eq!(output_payment.token_nonce, mb_token_expected_nonce);
                    assert_eq!(
                        output_payment.amount,
                        managed_biguint!(mb_token_expected_amount)
                    );
                    mb_token_nonce = output_payment.token_nonce;
                },
            )
            .assert_ok();

        mb_token_nonce
    }

    pub fn open_mystery_box(&mut self, mb_token_nonce: u64) {
        self.b_mock
            .execute_esdt_transfer(
                &self.user_address,
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
}
