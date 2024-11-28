use multiversx_sc::{
    imports::MultiValue6,
    types::{
        BigUint, EgldOrEsdtTokenIdentifier, EsdtLocalRole, ManagedBuffer, MultiValueEncoded,
        ReturnsResult, TestAddress, TestEsdtTransfer, TestSCAddress, TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, managed_buffer, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};
use mystery_box::{mystery_box_proxy, MysteryBox};

pub const CODE_PATH: MxscPath = MxscPath::new("output/mystery-box.mxsc.json");
pub const MB_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("MBTOK-abcdef");
pub const MYSTERY_BOX_ADDRESS: TestSCAddress = TestSCAddress::new("mystery-box");
pub const OWNER: TestAddress = TestAddress::new("owner");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/mystery-box");
    blockchain.register_contract(CODE_PATH, mystery_box::ContractBuilder);
    blockchain
}

pub struct MysteryBoxSetup {
    pub world: ScenarioWorld,
}

impl Default for MysteryBoxSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl MysteryBoxSetup {
    pub fn new() -> Self {
        let mut mystery_box = MysteryBoxSetup { world: world() };

        mystery_box.world.account(OWNER).nonce(1);

        let mb_token_roles = vec![
            EsdtLocalRole::NftCreate.name().to_string(),
            EsdtLocalRole::NftAddQuantity.name().to_string(),
            EsdtLocalRole::NftBurn.name().to_string(),
        ];
        mystery_box
            .world
            .account(MYSTERY_BOX_ADDRESS)
            .code(CODE_PATH)
            .owner(OWNER)
            .esdt_roles(MB_TOKEN, mb_token_roles);
        mystery_box
            .world
            .tx()
            .from(OWNER)
            .to(MYSTERY_BOX_ADDRESS)
            .whitebox(mystery_box::contract_obj, |sc| {
                sc.init(MB_TOKEN.to_token_identifier());
            });

        mystery_box.setup_mystery_box(900, 5_999, 1, 3_000, 0, 1_500, 1_000, 0, 50, 1, 1, 1, 0, 0);

        mystery_box
    }

    #[allow(clippy::too_many_arguments)]
    pub fn setup_mystery_box(
        &mut self,
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
        custom_reward_amount: u64,
        custom_reward_percentage: u64,
        custom_reward_cooldown: u64,
    ) {
        let mut rewards_list: MultiValueEncoded<StaticApi, MultiValue6<_, _, _, _, _, _>> =
            MultiValueEncoded::new();
        let mut reward = (
            mystery_box_proxy::RewardType::ExperiencePoints,
            EgldOrEsdtTokenIdentifier::egld(),
            BigUint::from(experience_points_amount),
            ManagedBuffer::from(b"ExperiencePoints"),
            experience_points_percentage,
            experience_points_cooldown,
        )
            .into();
        rewards_list.push(reward);

        reward = (
            mystery_box_proxy::RewardType::MysteryBox,
            EgldOrEsdtTokenIdentifier::esdt(MB_TOKEN.to_token_identifier()),
            BigUint::from(1u64),
            ManagedBuffer::from(b"MysteryBox"),
            sft_reward_percentage,
            sft_reward_cooldown,
        )
            .into();
        rewards_list.push(reward);

        reward = (
            mystery_box_proxy::RewardType::PercentValue,
            EgldOrEsdtTokenIdentifier::egld(),
            BigUint::from(percent_reward_amount),
            ManagedBuffer::from(b"Percent"),
            percent_reward_percentage,
            percent_reward_cooldown,
        )
            .into();
        rewards_list.push(reward);

        reward = (
            mystery_box_proxy::RewardType::FixedValue,
            EgldOrEsdtTokenIdentifier::egld(),
            BigUint::from(fixed_value_reward_amount),
            ManagedBuffer::from(b"FixedValue"),
            fixed_value_reward_percentage,
            fixed_value_reward_cooldown,
        )
            .into();
        rewards_list.push(reward);

        reward = (
            mystery_box_proxy::RewardType::CustomReward,
            EgldOrEsdtTokenIdentifier::egld(),
            BigUint::from(custom_reward_amount),
            ManagedBuffer::from(b"CustomText"),
            custom_reward_percentage,
            custom_reward_cooldown,
        )
            .into();
        rewards_list.push(reward);

        self.world
            .tx()
            .from(OWNER)
            .to(MYSTERY_BOX_ADDRESS)
            .typed(mystery_box_proxy::MysteryBoxProxy)
            .setup_mystery_box(rewards_list)
            .run();

        let mut uris = MultiValueEncoded::new();
        uris.push(managed_buffer!(b"www.cool_nft.com/my_nft.jpg"));
        uris.push(managed_buffer!(b"www.cool_nft.com/my_nft.json"));
        self.world
            .tx()
            .from(OWNER)
            .to(MYSTERY_BOX_ADDRESS)
            .typed(mystery_box_proxy::MysteryBoxProxy)
            .update_mystery_box_uris(uris)
            .run();
    }

    pub fn create_mystery_box(&mut self, amount: u64) -> u64 {
        let token = self
            .world
            .tx()
            .from(OWNER)
            .to(MYSTERY_BOX_ADDRESS)
            .typed(mystery_box_proxy::MysteryBoxProxy)
            .create_mystery_box(amount)
            .returns(ReturnsResult)
            .run();

        assert_eq!(token.amount, BigUint::from(amount));

        token.token_nonce
    }

    pub fn open_mystery_box(&mut self, mb_token_nonce: u64) {
        self.world
            .tx()
            .from(OWNER)
            .to(MYSTERY_BOX_ADDRESS)
            .typed(mystery_box_proxy::MysteryBoxProxy)
            .open_mystery_box()
            .esdt(TestEsdtTransfer(MB_TOKEN, mb_token_nonce, 1))
            .run();
    }
}
