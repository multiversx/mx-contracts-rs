multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const SFT_AMOUNT: u64 = 1;

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub enum RewardType {
    None,
    ExperiencePoints,
    MysteryBox,
    Percent,
    FixedValue,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub struct Reward<M: ManagedTypeApi> {
    pub reward_type: RewardType,
    pub reward_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub value: BigUint<M>,
    pub percentage_chance: u64,
    pub epochs_cooldown: u64,
}

impl<M: ManagedTypeApi> Default for Reward<M> {
    fn default() -> Self {
        Self {
            reward_type: RewardType::None,
            reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
            value: BigUint::zero(),
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
        percentage_chance: u64,
        epochs_cooldown: u64,
    ) -> Self {
        Reward {
            reward_type,
            reward_token_id,
            value,
            percentage_chance,
            epochs_cooldown,
        }
    }
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub struct MysteryBoxAttributes<M: ManagedTypeApi> {
    pub rewards: ManagedVec<M, Reward<M>>,
    pub create_epoch: u64,
}

impl<M: ManagedTypeApi> MysteryBoxAttributes<M> {
    #[inline]
    pub fn new(rewards: ManagedVec<M, Reward<M>>, create_epoch: u64) -> Self {
        MysteryBoxAttributes {
            rewards,
            create_epoch,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[view(getMysteryBoxTokenIdentifier)]
    #[storage_mapper("mysteryBoxTokenIdentifier")]
    fn mystery_box_token(&self) -> NonFungibleTokenMapper;

    #[view(getMysteryBoxCooldownPeriod)]
    #[storage_mapper("mysteryBoxCooldownPeriod")]
    fn mystery_box_cooldown_period(&self) -> SingleValueMapper<u64>;

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
