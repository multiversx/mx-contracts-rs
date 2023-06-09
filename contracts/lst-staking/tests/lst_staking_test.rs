use lst_staking::*;
use multiversx_sc::types::BigUint;
use multiversx_sc_scenario::DebugApi;

#[test]
fn test_add() {
    let _ = DebugApi::dummy();

    let lst_staking = lst_staking::contract_obj::<DebugApi>();

    lst_staking.init(10);
}
