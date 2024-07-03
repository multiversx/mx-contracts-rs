use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const SFT_AMOUNT: u64 = 1;
pub const ROYALTIES: u64 = 1_000;
pub const COLLECTION_NAME: &str = "Mystery Box";

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub enum RewardType {
    None,
    ExperiencePoints,
    MysteryBox,
    SFT,
    PercentValue,
    FixedValue,
    CustomReward,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub struct Reward<M: ManagedTypeApi> {
    pub reward_type: RewardType,
    pub reward_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub value: BigUint<M>,
    pub description: ManagedBuffer<M>,
    pub percentage_chance: u64,
    pub epochs_cooldown: u64,
}

impl<M: ManagedTypeApi> Default for Reward<M> {
    fn default() -> Self {
        Self {
            reward_type: RewardType::None,
            reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
            value: BigUint::zero(),
            description: ManagedBuffer::new(),
            percentage_chance: 0u64,
            epochs_cooldown: 0u64,
        }
    }
}

impl<M: ManagedTypeApi> Reward<M> {
    #[inline]
    pub fn new(
        reward_type: RewardType,
        reward_token_id: EgldOrEsdtTokenIdentifier<M>,
        value: BigUint<M>,
        description: ManagedBuffer<M>,
        percentage_chance: u64,
        epochs_cooldown: u64,
    ) -> Self {
        Reward {
            reward_type,
            reward_token_id,
            value,
            description,
            percentage_chance,
            epochs_cooldown,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[view(getMysteryBoxTokenIdentifier)]
    #[storage_mapper("mysteryBoxTokenIdentifier")]
    fn mystery_box_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGlobalCooldownEpoch)]
    #[storage_mapper("globalCooldownEpoch")]
    fn global_cooldown_epoch(&self, reward: &RewardType) -> SingleValueMapper<u64>;

    #[view(getWinningRates)]
    #[storage_mapper("winningRates")]
    fn winning_rates(&self) -> SingleValueMapper<ManagedVec<Reward<Self::Api>>>;

    #[view(getMysteryBoxUris)]
    #[storage_mapper("mysteryBoxUris")]
    fn mystery_box_uris(&self) -> SingleValueMapper<ManagedVec<ManagedBuffer>>;
}
