use liquid_locking::*;
use multiversx_sc::types::TokenIdentifier;
use multiversx_sc_scenario::{
    scenario_model::{
        Account, CheckAccount, CheckStateStep, IntoBlockchainCall, ScCallStep, SetStateStep,
        StepHandler, TxExpect,
    },
    ContractInfo, DebugApi, ScenarioWorld,
};

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/liquid-locking");

    blockchain.register_contract(
        "file:output/liquid-locking.wasm",
        liquid_locking::ContractBuilder,
    );
    blockchain
}

#[test]
fn test() {
    let _ = DebugApi::dummy();
    let mut world = world();
    let ic = world.interpreter_context();

    let owner_address = "address:owner";
    let user_1 = "address:user1";
    let user_2 = "address:user2";
    let mut contract = ContractInfo::<liquid_locking::Proxy<DebugApi>>::new("sc:liquid-locking");

    let whitelisted_token_1 = "str:AAA-111111";
    let whitelisted_token_2 = "str:BBB-222222";
    let blacklisted_token = "str:CCC-333333";

    world.set_state_step(
        SetStateStep::new()
            .put_account(owner_address, Account::new().nonce(1))
            .new_address(owner_address, 1, &contract),
    );

    // setup user accounts
    world
        .set_state_step(
            SetStateStep::new().put_account(
                user_1,
                Account::new()
                    .esdt_balance(whitelisted_token_1, 1_000u64)
                    .esdt_balance(whitelisted_token_2, 1_000u64),
            ),
        )
        .set_state_step(
            SetStateStep::new().put_account(
                user_2,
                Account::new()
                    .esdt_balance(blacklisted_token, 1_000u64)
                    .esdt_balance(whitelisted_token_2, 1_000u64),
            ),
        );

    // deploy
    let (new_address, ()) = contract
        .init(10u64)
        .into_blockchain_call()
        .from(owner_address)
        .contract_code("file:output/liquid-locking.wasm", &ic)
        .gas_limit("5,000,000")
        .expect(TxExpect::ok().no_result())
        .execute(&mut world);
    assert_eq!(new_address, contract.to_address());

    world.check_state_step(
        CheckStateStep::new()
            .put_account(owner_address, CheckAccount::new())
            .put_account(
                &contract,
                CheckAccount::new().check_storage("str:unbond_period", "10"),
            ),
    );

    // whitelist tokens

    world
        .sc_call_step(
            ScCallStep::new()
                .from(owner_address)
                .to(&contract)
                .call(contract.whitelist_token(TokenIdentifier::from(whitelisted_token_1)))
                .expect(TxExpect::ok().no_result()),
        )
        .sc_call_step(
            ScCallStep::new()
                .from(owner_address)
                .to(&contract)
                .call(contract.whitelist_token(TokenIdentifier::from(whitelisted_token_2)))
                .expect(TxExpect::ok().no_result()),
        )
        .check_state_step(
            CheckStateStep::new()
                .put_account(owner_address, CheckAccount::new())
                .put_account(
                    &contract,
                    CheckAccount::new()
                        .check_storage("str:unbond_period", "10")
                        .check_storage(
                            "str:token_whitelist",
                            "u32:2|nested:str:AAA-111111|nested:str:BBB-222222",
                        ),
                ),
        );

    // stake fail

    world.sc_call_step(
        ScCallStep::new()
            .from(user_2)
            .to(&contract)
            .esdt_transfer(blacklisted_token, 0u64, 500u64)
            .call(contract.stake())
            .expect(TxExpect::err(4, "str:token is not whitelisted")),
    );

    // stake success

    // world
    //     .sc_call_step(
    //         ScCallStep::new()
    //             .from(user_2)
    //             .to(&contract)
    //             .esdt_transfer(whitelisted_token_2, 0u64, 500u64)
    //             .call(contract.stake())
    //             .expect(TxExpect::ok().no_result()),
    //     )
    //     .sc_call_step(
    //         ScCallStep::new()
    //             .from(user_2)
    //             .to(&contract)
    //             .esdt_transfer(whitelisted_token_2, 0u64, 500u64)
    //             .call(contract.stake())
    //             .expect(TxExpect::ok().no_result()),
    //     )
    //     .sc_call_step(
    //         ScCallStep::new()
    //             .from(user_1)
    //             .to(&contract)
    //             .esdt_transfer(whitelisted_token_1, 0u64, 1_000u64)
    //             .esdt_transfer(whitelisted_token_2, 0u64, 1_000u64)
    //             .call(contract.stake())
    //             .expect(TxExpect::ok().no_result()),
    //     );
    // .check_state_step(
    //     CheckStateStep::new()
    //         .put_account(owner_address, CheckAccount::new())
    //         .put_account(user_1, CheckAccount::new())
    //         .put_account(user_2, CheckAccount::new())
    //         .put_account(
    //             contract,
    //             CheckAccount::new()
    //                 .esdt_balance(whitelisted_token_1, 1_000u64)
    //                 .esdt_balance(whitelisted_token_2, 2_000u64),
    //         ),
    // );
}
