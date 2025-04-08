use imports::{
    MxscPath, ReturnsResult, ReturnsResultUnmanaged, TestAddress, TestEsdtTransfer, TestSCAddress,
    TestTokenIdentifier,
};
use multiversx_sc::{
    codec::multi_types::OptionalValue,
    storage::mappers::SingleValue,
    types::{BigUint, ManagedAddress, MultiValueEncoded},
};
use multiversx_sc_scenario::{api::StaticApi, scenario_model::*, *};
use mvx_game_sc::game_proxy;

const GAME_SC_PATH: MxscPath = MxscPath::new("output/mvx-game-sc.mxsc.json");
const BALANCE: u64 = 100_000_000u64;
const TOKEN_GAME: TestTokenIdentifier = TestTokenIdentifier::new("GAME-123456");
const STARTING_FEE: u64 = 20u64;
const USER1_ADDR: TestAddress = TestAddress::new("user1");
const USER2_ADDR: TestAddress = TestAddress::new("user2");
const USER3_ADDR: TestAddress = TestAddress::new("user3");
const USER4_ADDR: TestAddress = TestAddress::new("user4");
const USER5_ADDR: TestAddress = TestAddress::new("user5");
const OWNER_ADDR: TestAddress = TestAddress::new("owner");
const GAME_SC_ADDR: TestSCAddress = TestSCAddress::new("mvx_game_sc");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(GAME_SC_PATH, mvx_game_sc::ContractBuilder);
    blockchain
}

struct GameContractState {
    world: ScenarioWorld,
}

impl GameContractState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace();

        world
            .account(OWNER_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        world
            .account(USER1_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        world
            .account(USER2_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        world
            .account(USER3_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        world
            .account(USER4_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        world
            .account(USER5_ADDR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(TOKEN_GAME, BALANCE);

        Self { world }
    }

    fn deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDR)
            .typed(game_proxy::MvxGameScProxy)
            .init(
                OptionalValue::Some(true),
                OptionalValue::Some(BigUint::from(STARTING_FEE)),
                OptionalValue::Some(TOKEN_GAME),
            )
            .code(GAME_SC_PATH)
            .new_address(GAME_SC_ADDR)
            .run();

        self
    }

    fn create_game(
        &mut self,
        waiting_time: u64,
        number_of_players_min: u64,
        number_of_players_max: u64,
        wager: u64,
        caller: TestAddress,
        expected_game_id: u64,
    ) -> &mut Self {
        self.world
            .tx()
            .from(caller)
            .to(GAME_SC_ADDR)
            .typed(game_proxy::MvxGameScProxy)
            .create_game(
                waiting_time,
                number_of_players_min,
                number_of_players_max,
                wager,
            )
            .esdt(TestEsdtTransfer(TOKEN_GAME, 0, STARTING_FEE))
            .with_result(ExpectValue(expected_game_id))
            .run();

        self
    }

    fn join_game(
        &mut self,
        game_id: u64,
        caller: TestAddress,
        amount: u64,
        expected_error: Option<(u64, &str)>,
    ) -> &mut Self {
        match expected_error {
            Some(val) => {
                self.world
                    .tx()
                    .from(caller)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .join_game(game_id)
                    .esdt(TestEsdtTransfer(TOKEN_GAME, 0, amount))
                    .with_result(ExpectError(val.0, val.1))
                    .run();
            }
            None => {
                self.world
                    .tx()
                    .from(caller)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .join_game(game_id)
                    .esdt(TestEsdtTransfer(TOKEN_GAME, 0, amount))
                    .run();
            }
        }

        self
    }

    fn claim_back_wager(
        &mut self,
        game_id: u64,
        caller: TestAddress,
        expected_error: Option<(u64, &str)>,
    ) -> &mut Self {
        match expected_error {
            Some(val) => {
                self.world
                    .tx()
                    .from(caller)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .claim_back_wager(game_id)
                    .with_result(ExpectError(val.0, val.1))
                    .run();
            }
            None => {
                self.world
                    .tx()
                    .from(caller)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .claim_back_wager(game_id)
                    .run();
            }
        }

        self
    }

    fn send_reward(
        &mut self,
        game_id: u64,
        winners: OptionalValue<MultiValueEncoded<StaticApi, (ManagedAddress<StaticApi>, u64)>>,
        expected_error: Option<(u64, &str)>,
    ) -> &mut Self {
        match expected_error {
            Some(val) => {
                self.world
                    .tx()
                    .from(OWNER_ADDR)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .send_reward(game_id, winners)
                    .with_result(ExpectError(val.0, val.1))
                    .run();
            }
            None => {
                self.world
                    .tx()
                    .from(OWNER_ADDR)
                    .to(GAME_SC_ADDR)
                    .typed(game_proxy::MvxGameScProxy)
                    .send_reward(game_id, winners)
                    .run();
            }
        }

        self
    }
}

#[test]
fn game_sc_deploy_test() {
    let mut state = GameContractState::new();

    state.deploy();
}

#[test]
fn game_sc_simple_game_flow() {
    let mut state = GameContractState::new();

    let waiting_time = 100u64;
    let number_of_players_min = 1u64;
    let number_of_players_max = 4u64;
    let wager = 100u64;

    // deploy
    state.deploy();

    // check last game id before creation
    state
        .world
        .query()
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .last_game_id()
        .run();

    // create first game
    state.create_game(
        waiting_time,
        number_of_players_min,
        number_of_players_max,
        wager,
        OWNER_ADDR,
        1u64,
    );

    // check last game id, needs to be 1
    state
        .world
        .query()
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .last_game_id()
        .with_result(ExpectValue(1u64))
        .run();

    // user1 tries to join the game, timestamp is ok, max players not reached, should work
    state.join_game(1u64, USER1_ADDR, wager, None);

    // min number of players reached, game should be valid
    let game_setting = state
        .world
        .tx()
        .from(USER1_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .game_settings(1u64)
        .returns(ReturnsResultUnmanaged)
        .run();
    let game_settings = SingleValue::from(game_setting);
    assert_eq!(game_settings.into().status, game_proxy::Status::Valid);

    // user2 tries to join the game, shuld work
    state.join_game(1u64, USER2_ADDR, wager, None);

    // both users should be in players vec
    let players = state
        .world
        .tx()
        .from(USER2_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .players(1u64)
        .returns(ReturnsResultUnmanaged)
        .run();
    let players_vec = players.0.to_vec();

    assert!(players_vec.contains(&USER1_ADDR.to_address()));
    assert!(players_vec.contains(&USER2_ADDR.to_address()));

    // game should be in users'storage
    let games_per_user1 = state
        .world
        .tx()
        .from(USER2_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .games_per_user(USER1_ADDR)
        .returns(ReturnsResultUnmanaged)
        .run();
    let games_per_user1_vec = games_per_user1.0.to_vec();

    let games_per_user2 = state
        .world
        .tx()
        .from(USER2_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .games_per_user(USER2_ADDR)
        .returns(ReturnsResultUnmanaged)
        .run();
    let games_per_user2_vec = games_per_user2.0.to_vec();

    assert!((games_per_user1_vec.contains(&1u64) && games_per_user2_vec.contains(&1u64)));
}

#[test]
fn game_sc_complex_flow() {
    let mut state = GameContractState::new();

    // game settings
    let waiting_time = 100u64; // => timestamp 102 should be out of waiting time
    let number_of_players_min = 2u64;
    let number_of_players_max = 4u64;
    let wager = 100u64;
    let diff_amount = 5u64;

    // deploy
    state.deploy();

    // set now = 1
    state.world.current_block().block_timestamp(1u64);

    // create first game
    state.create_game(
        waiting_time,
        number_of_players_min,
        number_of_players_max,
        wager,
        OWNER_ADDR,
        1,
    );

    // user1 joins
    state.join_game(1u64, USER1_ADDR, wager, None);

    let game_setting = state
        .world
        .tx()
        .from(USER1_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .game_settings(1u64)
        .returns(ReturnsResultUnmanaged)
        .run();
    let game_settings = SingleValue::from(game_setting);

    assert_eq!(game_settings.into().status, game_proxy::Status::Invalid);

    // user1 tries to claim back wager, should fail (waiting time not passed)
    state.claim_back_wager(1u64, USER1_ADDR, Some((4, "waiting time is not over yet")));

    // user2 joins
    state.join_game(
        1u64,
        USER2_ADDR,
        diff_amount,
        Some((4, "wrong amount paid")),
    ); // wrong amount paid

    state.join_game(1u64, USER2_ADDR, wager, None);
    state.join_game(
        1u64,
        USER2_ADDR,
        wager,
        Some((4, "user already joined this game")),
    ); // user already joined

    let game_setting = state
        .world
        .tx()
        .from(USER1_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .game_settings(1u64)
        .returns(ReturnsResult)
        .run();
    assert_eq!(game_setting.status, game_proxy::Status::Valid);

    // user3 joins
    state.join_game(1u64, USER3_ADDR, wager, None);

    // set timestamp after time limit
    state.world.current_block().block_timestamp(102u64);

    // user4 joins, time has passed, fails
    state.join_game(
        1u64,
        USER4_ADDR,
        wager,
        Some((4, "waiting time has passed")),
    );

    // user4 tries to claim back wager, fails
    state.claim_back_wager(
        1u64,
        USER4_ADDR,
        Some((4, "caller has not joined the game")),
    );

    // user1 tries to claim back wager, fails
    state.claim_back_wager(
        1u64,
        USER1_ADDR,
        Some((
            4,
            "can manually claim back wager only if the game is invalid",
        )),
    );

    // send tokens to sc
    state.world.transfer_step(
        TransferStep::new()
            .from(OWNER_ADDR)
            .to(GAME_SC_ADDR)
            .esdt_transfer(TOKEN_GAME.eval_to_expr(), 0, "10_000"),
    );

    state
        .world
        .check_account(GAME_SC_ADDR)
        .esdt_balance(TOKEN_GAME, 10_320u64);

    // owner sends rewards
    let mut winners = MultiValueEncoded::<StaticApi, (ManagedAddress<StaticApi>, u64)>::new();
    winners.push((ManagedAddress::from(USER1_ADDR.to_address()), 8000u64)); // 80%
    winners.push((ManagedAddress::from(USER2_ADDR.to_address()), 2000u64)); // 20%

    // make owner admin first
    state
        .world
        .tx()
        .from(OWNER_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .set_admin(OWNER_ADDR)
        .run();

    // send reward
    state.send_reward(1u64, OptionalValue::Some(winners), None);

    // user1 should receive 80% of the reward
    // reward = 3 * wager = 300 => user1's reward = 240
    state
        .world
        .check_account(USER1_ADDR)
        .esdt_balance(TOKEN_GAME, 100000140);

    // user2's reward = 60
    state
        .world
        .check_account(USER2_ADDR)
        .esdt_balance(TOKEN_GAME, 99999960); // balance - wager + 60
}

#[test]
fn invalid_game_test() {
    let mut state = GameContractState::new();

    // game settings
    let waiting_time = 100u64; // => timestamp 102 should be out of waiting time
    let number_of_players_min = 3u64;
    let number_of_players_max = 5u64;
    let wager = 100u64;

    // deploy
    state.deploy();

    // set now = 1
    state.world.current_block().block_timestamp(1);

    // create game
    state.create_game(
        waiting_time,
        number_of_players_min,
        number_of_players_max,
        wager,
        OWNER_ADDR,
        1,
    );

    // user1 joins
    state.join_game(1u64, USER1_ADDR, wager, None);

    // user2 joins
    state.join_game(1u64, USER2_ADDR, wager, None);

    // game is still invalid, min number of players not reached
    let game_setting = state
        .world
        .tx()
        .from(USER1_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .game_settings(1u64)
        .returns(ReturnsResult)
        .run();

    assert_eq!(game_setting.status, game_proxy::Status::Invalid);

    // set now = 102, past waiting time
    state.world.current_block().block_timestamp(102);

    // make owner admin first
    state
        .world
        .tx()
        .from(OWNER_ADDR)
        .to(GAME_SC_ADDR)
        .typed(game_proxy::MvxGameScProxy)
        .set_admin(OWNER_ADDR)
        .run();

    // send reward, invalid game => players should receive back wager, creator should receive the creation fee back
    state.send_reward(1u64, OptionalValue::None, None);

    state
        .world
        .check_account(USER1_ADDR)
        .esdt_balance(TOKEN_GAME, 100000000);

    state
        .world
        .check_account(USER2_ADDR)
        .esdt_balance(TOKEN_GAME, 100000000);

    state
        .world
        .check_account(OWNER_ADDR)
        .esdt_balance(TOKEN_GAME, 100000000);
}
