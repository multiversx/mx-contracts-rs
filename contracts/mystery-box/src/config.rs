multiversx_sc::imports!();
multiversx_sc::derive_imports!();

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
    pub reward_id: usize,
    pub reward_type: RewardType,
    pub reward_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub value: BigUint<M>,
    pub description: ManagedBuffer<M>,
    pub percentage_chance: u64,
}

impl<M: ManagedTypeApi> Default for Reward<M> {
    fn default() -> Self {
        Self {
            reward_id: 0usize,
            reward_type: RewardType::None,
            reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
            value: BigUint::zero(),
            description: ManagedBuffer::new(),
            percentage_chance: 0u64,
        }
    }
}

impl<M: ManagedTypeApi> Reward<M> {
    #[inline]
    pub fn new(
        reward_id: usize,
        reward_type: RewardType,
        reward_token_id: EgldOrEsdtTokenIdentifier<M>,
        value: BigUint<M>,
        description: ManagedBuffer<M>,
        percentage_chance: u64,
    ) -> Self {
        Reward {
            reward_id,
            reward_type,
            reward_token_id,
            value,
            description,
            percentage_chance,
        }
    }
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub enum CooldownType {
    None,
    Lifetime,
    ResetOnCooldown,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub struct RewardCooldown {
    pub cooldown_type: CooldownType,
    pub wins_per_cooldown: u64,
    pub cooldown_epochs: u64,
    pub remaining_epoch_wins: u64,
    pub last_update_epoch: u64,
}

impl Default for RewardCooldown {
    fn default() -> Self {
        Self {
            cooldown_type: CooldownType::Lifetime,
            wins_per_cooldown: 0u64,
            cooldown_epochs: 0u64,
            remaining_epoch_wins: 0u64,
            last_update_epoch: 0u64,
        }
    }
}

impl RewardCooldown {
    #[inline]
    pub fn new(
        cooldown_type: CooldownType,
        wins_per_cooldown: u64,
        cooldown_epochs: u64,
        remaining_epoch_wins: u64,
        last_update_epoch: u64,
    ) -> Self {
        RewardCooldown {
            cooldown_type,
            wins_per_cooldown,
            cooldown_epochs,
            remaining_epoch_wins,
            last_update_epoch,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[view(getMysteryBoxTokenIdentifier)]
    #[storage_mapper("mysteryBoxTokenIdentifier")]
    fn mystery_box_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getLastRewardId)]
    #[storage_mapper("lastRewardId")]
    fn last_reward_id(&self) -> SingleValueMapper<usize>;

    #[view(getRewardCooldown)]
    #[storage_mapper("rewardCooldown")]
    fn reward_cooldown(&self, reward_id: usize) -> SingleValueMapper<RewardCooldown>;

    #[view(getWinningRates)]
    #[storage_mapper("winningRates")]
    fn winning_rates(&self) -> SingleValueMapper<ManagedVec<Reward<Self::Api>>>;

    #[view(getMysteryBoxUris)]
    #[storage_mapper("mysteryBoxUris")]
    fn mystery_box_uris(&self) -> SingleValueMapper<ManagedVec<ManagedBuffer>>;
}
