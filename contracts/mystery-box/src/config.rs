multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const NFT_AMOUNT: u64 = 1;

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub enum RewardType {
    None,
    ExperiencePoints,
    NFT,
    Percent,
    FixedValue,
}

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone,
)]
pub struct Reward<M: ManagedTypeApi> {
    pub reward_type: RewardType,
    pub reward_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub reward_token_nonce: u64,
    pub value: BigUint<M>,
    pub percentage_chance: u64,
    pub epochs_cooldown: u64,
}

impl<M: ManagedTypeApi> Default for Reward<M> {
    fn default() -> Self {
        Self {
            reward_type: RewardType::None,
            reward_token_id: EgldOrEsdtTokenIdentifier::egld(),
            reward_token_nonce: 0u64,
            value: BigUint::zero(),
            percentage_chance: 0u64,
            epochs_cooldown: 0u64,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[view(getMysteryBoxTokenIdentifier)]
    #[storage_mapper("mysteryBoxTokenIdentifier")]
    fn mystery_box_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getCooldownEpoch)]
    #[storage_mapper("cooldownEpoch")]
    fn cooldown_epoch(&self) -> SingleValueMapper<u64>;
}
