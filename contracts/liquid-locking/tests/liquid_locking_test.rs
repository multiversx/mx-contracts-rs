use liquid_locking::*;
use multiversx_sc::types::{
    BigUint, EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ManagedVec, TestAddress, TestSCAddress,
    TestTokenIdentifier, TokenIdentifier,
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioWorld,
};

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const LIQUID_STAKING_ADDRESS: TestSCAddress = TestSCAddress::new("liquid-locking");
const CODE_PATH: MxscPath = MxscPath::new("output/liquid-locking.mxsc.json");
const FIRST_USER_ADDRESS: TestAddress = TestAddress::new("user1");
const SECOND_USER_ADDRESS: TestAddress = TestAddress::new("user2");
const WHITELIST_TOKEN_1: TestTokenIdentifier = TestTokenIdentifier::new("AAA-111111");
const WHITELIST_TOKEN_1_ID: &[u8] = b"AAA-111111";
const WHITELIST_TOKEN_2: TestTokenIdentifier = TestTokenIdentifier::new("BBB-222222");
const WHITELIST_TOKEN_2_ID: &[u8] = b"BBB-222222";
const BLACKLIST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("CCC-333333");
const BLACKLIST_TOKEN_ID: &[u8] = b"CCC-333333";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CODE_PATH, liquid_locking::ContractBuilder);
    blockchain
}

#[test]
fn test() {
    let mut world = world();

    world
        .account(OWNER_ADDRESS)
        .nonce(1)
        .new_address(OWNER_ADDRESS, 1, LIQUID_STAKING_ADDRESS);

    // setup user accounts

    world
        .account(FIRST_USER_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_1, 1000)
        .esdt_balance(WHITELIST_TOKEN_2, 1000);

    world
        .account(SECOND_USER_ADDRESS)
        .balance(1_000_000_000)
        .esdt_balance(BLACKLIST_TOKEN, 1000)
        .esdt_balance(WHITELIST_TOKEN_2, 1000);

    // deploy

    world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .init(10u64)
        .code(CODE_PATH)
        .new_address(LIQUID_STAKING_ADDRESS)
        .run();

    world.check_account(OWNER_ADDRESS);
    world
        .check_account(LIQUID_STAKING_ADDRESS)
        .check_storage("str:unbond_period", "10");

    // whitelist tokens

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .whitelist_token(WHITELIST_TOKEN_1_ID)
        .run();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .whitelist_token(WHITELIST_TOKEN_2_ID)
        .run();

    world.check_account(OWNER_ADDRESS);
    world
        .check_account(FIRST_USER_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_1, 1000)
        .esdt_balance(WHITELIST_TOKEN_2, 1000);
    world
        .check_account(SECOND_USER_ADDRESS)
        .balance(1_000_000_000)
        .esdt_balance(BLACKLIST_TOKEN, 1000)
        .esdt_balance(WHITELIST_TOKEN_2, 1000);

    world
        .check_account(LIQUID_STAKING_ADDRESS)
        .check_storage("str:unbond_period", "10")
        .check_storage("str:token_whitelist.len", "2")
        .check_storage("str:token_whitelist.item|u32:1", "str:AAA-111111")
        .check_storage("str:token_whitelist.item|u32:2", "str:BBB-222222")
        .check_storage("str:token_whitelist.index|nested:str:AAA-111111", "1")
        .check_storage("str:token_whitelist.index|nested:str:BBB-222222", "2");

    // lock fail

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BLACKLIST_TOKEN_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .with_result(ExpectError(4, "token is not whitelisted"))
        .run();

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld(1_000_000)
        .with_result(ExpectError(4, "no payment provided"))
        .run();

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .with_result(ExpectError(4, "no payment provided"))
        .run();

    // lock success

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WHITELIST_TOKEN_2_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WHITELIST_TOKEN_2_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WHITELIST_TOKEN_1_ID),
            0u64,
            &BigUint::from(1000u64),
        )
        .run();

    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .lock()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WHITELIST_TOKEN_2_ID),
            0u64,
            &BigUint::from(1000u64),
        )
        .run();

    world.check_account(OWNER_ADDRESS);
    world.check_account(FIRST_USER_ADDRESS);
    world
        .check_account(SECOND_USER_ADDRESS)
        .esdt_balance(BLACKLIST_TOKEN, 1000)
        .balance(1_000_000_000);
    world
        .check_account(LIQUID_STAKING_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_1, 1000)
        .esdt_balance(WHITELIST_TOKEN_2, 2000)
        .check_storage("str:unbond_period", "10")
        .check_storage("str:token_whitelist.len", "2")
        .check_storage("str:token_whitelist.item|u32:1", "str:AAA-111111")
        .check_storage("str:token_whitelist.item|u32:2", "str:BBB-222222")
        .check_storage("str:token_whitelist.index|nested:str:AAA-111111", "1")
        .check_storage("str:token_whitelist.index|nested:str:BBB-222222", "2")
        .check_storage("str:locked_tokens|address:user2|str:.len", "1")
        .check_storage(
            "str:locked_tokens|address:user2|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user2|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage(
            "str:locked_token_amounts|address:user2|nested:str:BBB-222222",
            "1000",
        )
        .check_storage("str:locked_tokens|address:user1|str:.len", "2")
        .check_storage(
            "str:locked_tokens|address:user1|str:.item|u32:1",
            "str:AAA-111111",
        )
        .check_storage(
            "str:locked_tokens|address:user1|str:.item|u32:2",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user1|str:.index|nested:str:AAA-111111",
            "1",
        )
        .check_storage(
            "str:locked_tokens|address:user1|str:.index|nested:str:BBB-222222",
            "2",
        )
        .check_storage(
            "str:locked_token_amounts|address:user1|nested:str:AAA-111111",
            "1000",
        )
        .check_storage(
            "str:locked_token_amounts|address:user1|nested:str:BBB-222222",
            "1000",
        );

    // unstake fail

    let mut unlock_single_esdt = ManagedVec::<StaticApi, EsdtTokenPayment<StaticApi>>::new();
    let mut unlock_multiple_esdt = ManagedVec::<StaticApi, EsdtTokenPayment<StaticApi>>::new();

    unlock_single_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(BLACKLIST_TOKEN_ID),
        token_nonce: 0,
        amount: BigUint::from(1500u64),
    });
    unlock_multiple_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(WHITELIST_TOKEN_1_ID),
        token_nonce: 0,
        amount: BigUint::zero(),
    });
    unlock_multiple_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(WHITELIST_TOKEN_1_ID),
        token_nonce: 0,
        amount: BigUint::from(300u64),
    });

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_single_esdt)
        .with_result(ExpectError(4, "unavailable amount"))
        .run();

    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_multiple_esdt)
        .with_result(ExpectError(4, "requested amount cannot be 0"))
        .run();

    // unlock success

    unlock_single_esdt = ManagedVec::<StaticApi, EsdtTokenPayment<StaticApi>>::new();
    unlock_multiple_esdt = ManagedVec::<StaticApi, EsdtTokenPayment<StaticApi>>::new();

    unlock_single_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(WHITELIST_TOKEN_2_ID),
        token_nonce: 0,
        amount: BigUint::from(200u64),
    });
    unlock_multiple_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(WHITELIST_TOKEN_1_ID),
        token_nonce: 0,
        amount: BigUint::from(1000u64),
    });
    unlock_multiple_esdt.push(EsdtTokenPayment {
        token_identifier: TokenIdentifier::from(WHITELIST_TOKEN_2_ID),
        token_nonce: 0,
        amount: BigUint::from(300u64),
    });
    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_single_esdt.clone())
        .run();

    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_multiple_esdt)
        .run();

    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_single_esdt.clone())
        .run();

    world.current_block().block_epoch(8);
    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unlock(unlock_single_esdt)
        .run();

    world.check_account(OWNER_ADDRESS);
    world.check_account(FIRST_USER_ADDRESS);
    world
        .check_account(SECOND_USER_ADDRESS)
        .esdt_balance(BLACKLIST_TOKEN, 1000)
        .balance(1_000_000_000);
    world
        .check_account(LIQUID_STAKING_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_1, 1_000u64)
        .esdt_balance(WHITELIST_TOKEN_2, 2_000u64)
        .check_storage("str:unbond_period", "10")
        .check_storage("str:token_whitelist.len", "2")
        .check_storage("str:token_whitelist.item|u32:1", "str:AAA-111111")
        .check_storage("str:token_whitelist.item|u32:2", "str:BBB-222222")
        .check_storage("str:token_whitelist.index|nested:str:AAA-111111", "1")
        .check_storage("str:token_whitelist.index|nested:str:BBB-222222", "2")
        .check_storage("str:locked_tokens|address:user2|str:.len", "1")
        .check_storage(
            "str:locked_tokens|address:user2|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user2|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage(
            "str:locked_token_amounts|address:user2|nested:str:BBB-222222",
            "800",
        )
        .check_storage("str:locked_tokens|address:user1|str:.len", "1")
        .check_storage(
            "str:locked_tokens|address:user1|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user1|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage(
            "str:locked_token_amounts|address:user1|nested:str:BBB-222222",
            "300",
        )
        .check_storage("str:unlocked_tokens|address:user2|str:.len", "1")
        .check_storage(
            "str:unlocked_tokens|address:user2|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:unlocked_tokens|address:user2|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage("str:unlocked_tokens|address:user1|str:.len", "2")
        .check_storage(
            "str:unlocked_tokens|address:user1|str:.item|u32:1",
            "str:AAA-111111",
        )
        .check_storage(
            "str:unlocked_tokens|address:user1|str:.item|u32:2",
            "str:BBB-222222",
        )
        .check_storage(
            "str:unlocked_tokens|address:user1|str:.index|nested:str:AAA-111111",
            "1",
        )
        .check_storage(
            "str:unlocked_tokens|address:user1|str:.index|nested:str:BBB-222222",
            "2",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:BBB-222222|str:.len",
            "2",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:BBB-222222|str:.item|u32:1",
            "10",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:BBB-222222|str:.item|u32:2",
            "18",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:BBB-222222|str:.index|u64:10",
            "1",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:BBB-222222|str:.index|u64:18",
            "2",
        )
        .check_storage(
            "str:unlocked_token_amounts|address:user1|nested:str:BBB-222222|u64:10",
            "500",
        )
        .check_storage(
            "str:unlocked_token_amounts|address:user1|nested:str:BBB-222222|u64:18",
            "200",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:AAA-111111|str:.len",
            "1",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:AAA-111111|str:.item|u32:1",
            "10",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user1|nested:str:AAA-111111|str:.index|u64:10",
            "1",
        )
        .check_storage(
            "str:unlocked_token_amounts|address:user1|nested:str:AAA-111111|u64:10",
            "1000",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user2|nested:str:BBB-222222|str:.len",
            "1",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user2|nested:str:BBB-222222|str:.item|u32:1",
            "10",
        )
        .check_storage(
            "str:unlocked_token_epochs|address:user2|nested:str:BBB-222222|str:.index|u64:10",
            "1",
        )
        .check_storage(
            "str:unlocked_token_amounts|address:user2|nested:str:BBB-222222|u64:10",
            "200",
        );

    // unbond fail

    let mut unbond_tokens = ManagedVec::<StaticApi, TokenIdentifier<StaticApi>>::new();
    unbond_tokens.push(TokenIdentifier::from(BLACKLIST_TOKEN_ID));
    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unbond(unbond_tokens)
        .with_result(ExpectError(4, "nothing to unbond"))
        .run();

    unbond_tokens = ManagedVec::<StaticApi, TokenIdentifier<StaticApi>>::new();
    unbond_tokens.push(TokenIdentifier::from(WHITELIST_TOKEN_2_ID));
    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unbond(unbond_tokens)
        .with_result(ExpectError(4, "nothing to unbond"))
        .run();

    // unbond success

    unbond_tokens = ManagedVec::<StaticApi, TokenIdentifier<StaticApi>>::new();
    unbond_tokens.push(TokenIdentifier::from(WHITELIST_TOKEN_2_ID));
    world.current_block().block_epoch(11);
    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unbond(unbond_tokens)
        .run();

    unbond_tokens = ManagedVec::<StaticApi, TokenIdentifier<StaticApi>>::new();
    unbond_tokens.push(TokenIdentifier::from(WHITELIST_TOKEN_2_ID));
    world.current_block().block_epoch(22);
    world
        .tx()
        .from(SECOND_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unbond(unbond_tokens)
        .run();

    unbond_tokens = ManagedVec::<StaticApi, TokenIdentifier<StaticApi>>::new();
    unbond_tokens.push(TokenIdentifier::from(WHITELIST_TOKEN_2_ID));
    unbond_tokens.push(TokenIdentifier::from(WHITELIST_TOKEN_1_ID));
    world
        .tx()
        .from(FIRST_USER_ADDRESS)
        .to(LIQUID_STAKING_ADDRESS)
        .typed(liquid_locking_proxy::LiquidLockingProxy)
        .unbond(unbond_tokens)
        .run();

    world.check_account(OWNER_ADDRESS);
    world
        .check_account(FIRST_USER_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_1, 1_000)
        .esdt_balance(WHITELIST_TOKEN_2, 700);

    world
        .check_account(SECOND_USER_ADDRESS)
        .esdt_balance(BLACKLIST_TOKEN, 1_000)
        .esdt_balance(WHITELIST_TOKEN_2, 200)
        .balance(1_000_000_000);

    world
        .check_account(LIQUID_STAKING_ADDRESS)
        .esdt_balance(WHITELIST_TOKEN_2, 1_100)
        .check_storage("str:unbond_period", "10")
        .check_storage("str:token_whitelist.len", "2")
        .check_storage("str:token_whitelist.item|u32:1", "str:AAA-111111")
        .check_storage("str:token_whitelist.item|u32:2", "str:BBB-222222")
        .check_storage("str:token_whitelist.index|nested:str:AAA-111111", "1")
        .check_storage("str:token_whitelist.index|nested:str:BBB-222222", "2")
        .check_storage("str:locked_tokens|address:user2|str:.len", "1")
        .check_storage(
            "str:locked_tokens|address:user2|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user2|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage(
            "str:locked_token_amounts|address:user2|nested:str:BBB-222222",
            "800",
        )
        .check_storage("str:locked_tokens|address:user1|str:.len", "1")
        .check_storage(
            "str:locked_tokens|address:user1|str:.item|u32:1",
            "str:BBB-222222",
        )
        .check_storage(
            "str:locked_tokens|address:user1|str:.index|nested:str:BBB-222222",
            "1",
        )
        .check_storage(
            "str:locked_token_amounts|address:user1|nested:str:BBB-222222",
            "300",
        );
}
