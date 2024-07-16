use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, Debug, PartialEq,
)]
pub enum Status {
    Valid,
    Invalid,
}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, Debug, PartialEq,
)]
pub struct GameSettings<M: ManagedTypeApi> {
    pub time_limit: u64,            //start_time + waiting time
    pub number_of_players_min: u64, //min and max
    pub number_of_players_max: u64,
    pub wager: BigUint<M>,
    pub creator: ManagedAddress<M>,
    pub status: Status,
}
