#![allow(dead_code)]
use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Debug, PartialEq)]
pub struct GameConfig<M: ManagedTypeApi> {
    pub signing_server_address: ManagedAddress<M>,
    pub podium_size: u32,
    pub prize_distribution_percentages: ManagedVec<M, u32>, // percentages in basis points (e.g., [5000, 3000, 2000] for 50.00%, 30.00%, 20.00%)
    pub house_fee_percentage: u32,                          // in basis points (e.g., 1234 = 12.34%)
    pub allow_late_join: bool,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Debug, PartialEq)]
pub enum TournamentStatus {
    Joining,
    Playing,
    ProcessingResults,
    Completed,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Debug, PartialEq)]
pub struct Tournament<M: ManagedTypeApi> {
    pub game_id: ManagedBuffer<M>,
    pub status: TournamentStatus,
    pub entry_fee: BigUint<M>,
    pub participants: ManagedVec<M, ManagedAddress<M>>,
    pub prize_pool: BigUint<M>,
    pub join_deadline: u64,
    pub play_deadline: u64,
    pub final_podium: ManagedVec<M, ManagedAddress<M>>,
    pub creator: ManagedAddress<M>,
}

#[type_abi]
#[derive(
    TopEncode, TopDecode, ManagedVecItem, NestedEncode, NestedDecode, Clone, Debug, PartialEq,
)]
pub struct SpectatorBet<M: ManagedTypeApi> {
    pub bettor_address: ManagedAddress<M>,
    pub amount: BigUint<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, Clone, Debug, PartialEq)]
pub struct SpectatorClaim<M: ManagedTypeApi> {
    pub tournament_id: ManagedBuffer<M>,
    pub bettor_address: ManagedAddress<M>,
    pub has_claimed: bool,
}
