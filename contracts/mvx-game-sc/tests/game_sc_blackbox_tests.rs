use multiversx_sc::{
    codec::{multi_types::OptionalValue, CodecFrom},
    storage::mappers::SingleValue,
    types::{Address, EgldOrEsdtTokenIdentifier, TokenIdentifier},
};
use multiversx_sc_scenario::{api::StaticApi, num_bigint::BigUint, scenario_model::*, *};
use mvx_game_sc::{
    storage::{ProxyTrait, StorageModule},
    types::{GameSettings, Status},
    ProxyTrait as _,
};

const GAME_SC_PATH: &str = "file:output/mvx-game-sc.wasm";
const BALANCE: u64 = 100_000_000u64;
const TOKEN_ID: &str = "str:GAME-123456";
const TOKEN_ID_BY: &[u8] = b"GAME-123456";
const STARTING_FEE: u64 = 20u64;
const USER1_ADDR: &str = "address:user1";
const USER2_ADDR: &str = "address:user2";
const OWNER_ADDR: &str = "address:owner";
const GAME_SC_ADDR: &str = "sc:mvx_game_sc";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/mvx-game-sc");

    blockchain.register_contract(GAME_SC_PATH, mvx_game_sc::ContractBuilder);
    blockchain
}

struct GameContractState {
    world: ScenarioWorld,
    user1: AddressValue,
    user2: AddressValue,
    owner: AddressValue,
}

impl GameContractState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace().set_state_step(
            SetStateStep::new()
                .put_account(
                    OWNER_ADDR,
                    Account::new()
                        .nonce(1)
                        .balance(BALANCE)
                        .esdt_balance(TOKEN_ID, BALANCE),
                )
                .put_account(
                    USER1_ADDR,
                    Account::new()
                        .nonce(2)
                        .balance(BALANCE)
                        .esdt_balance(TOKEN_ID, BALANCE),
                )
                .put_account(
                    USER2_ADDR,
                    Account::new()
                        .nonce(3)
                        .balance(BALANCE)
                        .esdt_balance(TOKEN_ID, BALANCE),
                ),
        );

        let user1 = AddressValue::from(USER1_ADDR);
        let user2 = AddressValue::from(USER2_ADDR);
        let owner = AddressValue::from(OWNER_ADDR);

        Self {
            world,
            user1,
            user2,
            owner,
        }
    }

    fn deploy(&mut self, game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>) -> &mut Self {
        let game_sc_code = self.world.code_expression(GAME_SC_PATH);

        self.world
            .set_state_step(SetStateStep::new().new_address(OWNER_ADDR, 1, GAME_SC_ADDR))
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER_ADDR)
                    .code(game_sc_code)
                    .call(game_sc.init(
                        OptionalValue::Some(true),
                        OptionalValue::Some(BigUint::from(STARTING_FEE)),
                        OptionalValue::Some(EgldOrEsdtTokenIdentifier::esdt(
                            TokenIdentifier::from(TOKEN_ID_BY),
                        )),
                    ))
                    .expect(TxExpect::ok().no_result()),
            );

        self
    }

    fn create_game(
        &mut self,
        waiting_time: u64,
        number_of_players_min: u64,
        number_of_players_max: u64,
        wager: BigUint,
        caller: &str,
        game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
    ) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new()
                .from(caller)
                .to(GAME_SC_ADDR)
                .esdt_transfer(TOKEN_ID, 0u64, BigUint::from(STARTING_FEE))
                .call(game_sc.create_game(
                    waiting_time,
                    number_of_players_min,
                    number_of_players_max,
                    wager,
                ))
                .expect(TxExpect::ok().no_result()),
        );

        self
    }

    fn join_game(
        &mut self,
        game_id: u64,
        caller: &str,
        amount: &BigUint,
        game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
    ) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new()
                .from(caller)
                .to(GAME_SC_ADDR)
                .esdt_transfer(TOKEN_ID, 0u64, amount)
                .call(game_sc.join_game(game_id))
                .expect(TxExpect::ok().no_result()),
        );
        self
    }
}

#[test]
fn game_sc_deploy_test() {
    let mut state = GameContractState::new();
    let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

    state.deploy(&mut game_sc);
}

#[test]
fn game_sc_simple_game_flow() {
    let mut state = GameContractState::new();
    let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

    let waiting_time = 100u64;
    let number_of_players_min = 1u64;
    let number_of_players_max = 4u64;
    let wager = BigUint::from(100u64);

    //deploy
    state.deploy(&mut game_sc);

    //check last game id before creation
    state.world.sc_query(
        ScQueryStep::new()
            .to(GAME_SC_ADDR)
            .function("getLastGameId")
            .expect(TxExpect::ok().result("")),
    );

    //create first game
    state.create_game(
        waiting_time,
        number_of_players_min,
        number_of_players_max,
        wager.clone(),
        OWNER_ADDR,
        &mut game_sc,
    );

    //check last game id, needs to be 1
    state.world.sc_query(
        ScQueryStep::new()
            .to(GAME_SC_ADDR)
            .function("getLastGameId")
            .expect(TxExpect::ok().result("1")),
    );

    //user1 tries to join the game, timestamp is ok, max players not reached, should work
    state.join_game(1u64, USER1_ADDR, &wager, &mut game_sc);

    //min number of players reached, game should be valid
    let game_settings: SingleValue<GameSettings<StaticApi>> = game_sc
        .game_settings(1u64)
        .into_vm_query()
        .expect(TxExpect::ok())
        .execute(&mut state.world);

    assert_eq!(game_settings.into().status, Status::Valid);
}

