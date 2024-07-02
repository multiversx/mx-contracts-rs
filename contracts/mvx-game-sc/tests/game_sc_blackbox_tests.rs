// TODO: Refactor to Unified Syntax

// use multiversx_sc::{
//     codec::multi_types::OptionalValue,
//     storage::mappers::SingleValue,
//     types::{
//         Address, EgldOrEsdtTokenIdentifier, ManagedAddress, MultiValueEncoded, TokenIdentifier,
//     },
// };
// use multiversx_sc_scenario::{api::StaticApi, scenario_model::*, *};
// use mvx_game_sc::{
//     owner::ProxyTrait,
//     storage::ProxyTrait as _,
//     types::{GameSettings, Status},
//     ProxyTrait as _,
// };

// use num_bigint::BigUint;

// const GAME_SC_PATH: &str = "mxsc:output/mvx-game-sc.mxsc.json";
// const BALANCE: u64 = 100_000_000u64;
// const TOKEN_ID: &str = "str:GAME-123456";
// const TOKEN_ID_BY: &[u8] = b"GAME-123456";
// const STARTING_FEE: u64 = 20u64;
// const USER1_ADDR: &str = "address:user1";
// const USER2_ADDR: &str = "address:user2";
// const USER3_ADDR: &str = "address:user3";
// const USER4_ADDR: &str = "address:user4";
// const USER5_ADDR: &str = "address:user5";
// const OWNER_ADDR: &str = "address:owner";
// const GAME_SC_ADDR: &str = "sc:mvx_game_sc";

// fn world() -> ScenarioWorld {
//     let mut blockchain = ScenarioWorld::new();
//     blockchain.register_contract(GAME_SC_PATH, mvx_game_sc::ContractBuilder);
//     blockchain
// }

// struct GameContractState {
//     world: ScenarioWorld,
//     user1: Address,
//     user2: Address,
//     user3: Address,
//     user4: Address,
//     user5: Address,
//     owner: Address,
// }

// impl GameContractState {
//     fn new() -> Self {
//         let mut world = world();
//         world.start_trace().set_state_step(
//             SetStateStep::new()
//                 .put_account(
//                     OWNER_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 )
//                 .put_account(
//                     USER1_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 )
//                 .put_account(
//                     USER2_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 )
//                 .put_account(
//                     USER3_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 )
//                 .put_account(
//                     USER4_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 )
//                 .put_account(
//                     USER5_ADDR,
//                     Account::new()
//                         .nonce(1)
//                         .balance(BALANCE)
//                         .esdt_balance(TOKEN_ID, BALANCE),
//                 ),
//         );

//         let user1 = AddressValue::from(USER1_ADDR).to_address();
//         let user2 = AddressValue::from(USER2_ADDR).to_address();
//         let user3 = AddressValue::from(USER3_ADDR).to_address();
//         let user4 = AddressValue::from(USER4_ADDR).to_address();
//         let user5 = AddressValue::from(USER5_ADDR).to_address();
//         let owner = AddressValue::from(OWNER_ADDR).to_address();

//         Self {
//             world,
//             user1,
//             user2,
//             user3,
//             user4,
//             user5,
//             owner,
//         }
//     }

//     fn deploy(&mut self, game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>) -> &mut Self {
//         let game_sc_code = self.world.code_expression(GAME_SC_PATH);

//         self.world
//             .set_state_step(SetStateStep::new().new_address(OWNER_ADDR, 1, GAME_SC_ADDR))
//             .sc_deploy(
//                 ScDeployStep::new()
//                     .from(OWNER_ADDR)
//                     .code(game_sc_code)
//                     .call(game_sc.init(
//                         OptionalValue::Some(true),
//                         OptionalValue::Some(BigUint::from(STARTING_FEE)),
//                         OptionalValue::Some(EgldOrEsdtTokenIdentifier::esdt(
//                             TokenIdentifier::from(TOKEN_ID_BY),
//                         )),
//                     ))
//                     .expect(TxExpect::ok().no_result()),
//             );

//         self
//     }

//     fn create_game(
//         &mut self,
//         waiting_time: u64,
//         number_of_players_min: u64,
//         number_of_players_max: u64,
//         wager: BigUint,
//         caller: &str,
//         game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
//         expected_game_id: &str,
//     ) -> &mut Self {
//         self.world.sc_call(
//             ScCallStep::new()
//                 .from(caller)
//                 .to(GAME_SC_ADDR)
//                 .esdt_transfer(TOKEN_ID, 0u64, BigUint::from(STARTING_FEE))
//                 .call(game_sc.create_game(
//                     waiting_time,
//                     number_of_players_min,
//                     number_of_players_max,
//                     wager,
//                 ))
//                 .expect(TxExpect::ok().result(expected_game_id)),
//         );

//         self
//     }

//     fn join_game(
//         &mut self,
//         game_id: u64,
//         caller: &str,
//         amount: BigUint,
//         game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
//         expected_error: OptionalValue<(&str, &str)>,
//     ) -> &mut Self {
//         match expected_error {
//             OptionalValue::Some(val) => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(caller)
//                         .to(GAME_SC_ADDR)
//                         .esdt_transfer(TOKEN_ID, 0u64, amount)
//                         .call(game_sc.join_game(game_id))
//                         .expect(TxExpect::err(val.0, val.1)),
//                 );
//             }
//             OptionalValue::None => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(caller)
//                         .to(GAME_SC_ADDR)
//                         .esdt_transfer(TOKEN_ID, 0u64, amount)
//                         .call(game_sc.join_game(game_id))
//                         .expect(TxExpect::ok().no_result()),
//                 );
//             }
//         }

//         self
//     }

//     fn claim_back_wager(
//         &mut self,
//         game_id: u64,
//         caller: &str,
//         game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
//         expected_error: OptionalValue<(&str, &str)>,
//     ) -> &mut Self {
//         match expected_error {
//             OptionalValue::Some(val) => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(caller)
//                         .to(GAME_SC_ADDR)
//                         .call(game_sc.claim_back_wager(game_id))
//                         .expect(TxExpect::err(val.0, val.1)),
//                 );
//             }
//             OptionalValue::None => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(caller)
//                         .to(GAME_SC_ADDR)
//                         .call(game_sc.claim_back_wager(game_id))
//                         .expect(TxExpect::ok().no_result()),
//                 );
//             }
//         }

//         self
//     }

//     fn send_reward(
//         &mut self,
//         game_id: u64,
//         game_sc: &mut ContractInfo<mvx_game_sc::Proxy<StaticApi>>,
//         winners: OptionalValue<MultiValueEncoded<StaticApi, (ManagedAddress<StaticApi>, u64)>>,
//         expected_error: OptionalValue<(&str, &str)>,
//     ) -> &mut Self {
//         match expected_error {
//             OptionalValue::Some(val) => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(OWNER_ADDR)
//                         .to(GAME_SC_ADDR)
//                         .call(game_sc.send_reward(game_id, winners))
//                         .expect(TxExpect::err(val.0, val.1)),
//                 );
//             }
//             OptionalValue::None => {
//                 self.world.sc_call(
//                     ScCallStep::new()
//                         .from(OWNER_ADDR)
//                         .to(GAME_SC_ADDR)
//                         .call(game_sc.send_reward(game_id, winners))
//                         .expect(TxExpect::ok().no_result()),
//                 );
//             }
//         }

//         self
//     }
// }

// #[test]
// fn game_sc_deploy_test() {
//     let mut state = GameContractState::new();
//     let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

//     state.deploy(&mut game_sc);
// }

// #[test]
// fn game_sc_simple_game_flow() {
//     let mut state = GameContractState::new();
//     let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

//     let waiting_time = 100u64;
//     let number_of_players_min = 1u64;
//     let number_of_players_max = 4u64;
//     let wager = BigUint::from(100u64);
//     let user1 = state.user1.clone();
//     let user2 = state.user2.clone();
//     let _ = state.user3;
//     let _ = state.user4;
//     let _ = state.user5;
//     let _ = state.owner.clone();

//     //deploy
//     state.deploy(&mut game_sc);

//     //check last game id before creation
//     state.world.sc_query(
//         ScQueryStep::new()
//             .to(GAME_SC_ADDR)
//             .function("getLastGameId")
//             .expect(TxExpect::ok().result("")),
//     );

//     //create first game
//     state.create_game(
//         waiting_time,
//         number_of_players_min,
//         number_of_players_max,
//         wager.clone(),
//         OWNER_ADDR,
//         &mut game_sc,
//         "1",
//     );

//     //check last game id, needs to be 1
//     state.world.sc_query(
//         ScQueryStep::new()
//             .to(GAME_SC_ADDR)
//             .function("getLastGameId")
//             .expect(TxExpect::ok().result("1")),
//     );

//     //user1 tries to join the game, timestamp is ok, max players not reached, should work
//     state.join_game(
//         1u64,
//         USER1_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::None,
//     );

//     //min number of players reached, game should be valid
//     let game_settings: SingleValue<GameSettings<StaticApi>> = game_sc
//         .game_settings(1u64)
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);

//     assert_eq!(game_settings.into().status, Status::Valid);

//     //user2 tries to join the game, shuld work
//     state.join_game(1u64, USER2_ADDR, wager, &mut game_sc, OptionalValue::None);

//     //both users should be in players vec
//     let players: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> = game_sc
//         .players(1u64)
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);
//     let players_vec = players.to_vec();

//     assert!(players_vec.contains(&ManagedAddress::from(user1.clone())));
//     assert!(players_vec.contains(&ManagedAddress::from(user2.clone())));

//     //game should be in users'storage
//     let games_per_user1: MultiValueEncoded<StaticApi, u64> = game_sc
//         .games_per_user(&ManagedAddress::from(user1))
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);
//     let games_per_user1_vec = games_per_user1.to_vec();

//     let games_per_user2: MultiValueEncoded<StaticApi, u64> = game_sc
//         .games_per_user(&ManagedAddress::from(user2))
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);
//     let games_per_user2_vec = games_per_user2.to_vec();

//     assert!((games_per_user1_vec.contains(&1u64) && games_per_user2_vec.contains(&1u64)));
// }

// #[test]
// fn game_sc_complex_flow() {
//     let mut state = GameContractState::new();
//     let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

//     //game settings
//     let waiting_time = 100u64; // => timestamp 102 should be out of waiting time
//     let number_of_players_min = 2u64;
//     let number_of_players_max = 4u64;
//     let wager = BigUint::from(100u64);
//     let diff_amount = BigUint::from(5u64);

//     //users
//     let _user1 = state.user1.clone();
//     let _user2 = state.user2.clone();
//     let _user3 = state.user3.clone();
//     let _user4 = state.user4.clone();
//     let _user5 = state.user5.clone();
//     let _owner = state.owner.clone();

//     //deploy
//     state.deploy(&mut game_sc);

//     //set now = 1
//     state
//         .world
//         .set_state_step(SetStateStep::new().block_timestamp(1u64));

//     //create first game
//     state.create_game(
//         waiting_time,
//         number_of_players_min,
//         number_of_players_max,
//         wager.clone(),
//         OWNER_ADDR,
//         &mut game_sc,
//         "1",
//     );

//     //user1 joins
//     state.join_game(
//         1u64,
//         USER1_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::None,
//     );

//     let game_settings: SingleValue<GameSettings<StaticApi>> = game_sc
//         .game_settings(1u64)
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);

//     assert_eq!(game_settings.into().status, Status::Invalid);

//     //user1 tries to claim back wager, should fail (waiting time not passed)
//     state.claim_back_wager(
//         1u64,
//         USER1_ADDR,
//         &mut game_sc,
//         OptionalValue::Some(("4", "str:waiting time is not over yet")),
//     );

//     //user2 joins
//     state.join_game(
//         1u64,
//         USER2_ADDR,
//         diff_amount,
//         &mut game_sc,
//         OptionalValue::Some(("4", "str:wrong amount paid")),
//     ); //wrong amount paid

//     state.join_game(
//         1u64,
//         USER2_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::None,
//     );
//     state.join_game(
//         1u64,
//         USER2_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::Some(("4", "str:user already joined this game")),
//     ); //user already joined

//     let game_settings: SingleValue<GameSettings<StaticApi>> = game_sc
//         .game_settings(1u64)
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);

//     assert_eq!(game_settings.into().status, Status::Valid);

//     //user3 joins
//     state.join_game(
//         1u64,
//         USER3_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::None,
//     );

//     //set timestamp after time limit
//     state
//         .world
//         .set_state_step(SetStateStep::new().block_timestamp(102u64));

//     //user4 joins, time has passed, fails
//     state.join_game(
//         1u64,
//         USER4_ADDR,
//         wager,
//         &mut game_sc,
//         OptionalValue::Some(("4", "str:waiting time has passed")),
//     );

//     //user4 tries to claim back wager, fails
//     state.claim_back_wager(
//         1u64,
//         USER4_ADDR,
//         &mut game_sc,
//         OptionalValue::Some(("4", "str:caller has not joined the game")),
//     );

//     //user1 tries to claim back wager, fails
//     state.claim_back_wager(
//         1u64,
//         USER1_ADDR,
//         &mut game_sc,
//         OptionalValue::Some((
//             "4",
//             "str:can manually claim back wager only if the game is invalid",
//         )),
//     );

//     //send tokens to sc
//     state.world.transfer_step(
//         TransferStep::new()
//             .esdt_transfer(TOKEN_ID, 0u64, BigUint::from(10_000u64))
//             .from(OWNER_ADDR)
//             .to(GAME_SC_ADDR),
//     );

//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             GAME_SC_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, BigUint::from(10_320u64)),
//         ));

//     //owner sends rewards
//     let mut winners = MultiValueEncoded::<StaticApi, (ManagedAddress<StaticApi>, u64)>::new();
//     winners.push((ManagedAddress::from(_user1), 8000u64)); //80%
//     winners.push((ManagedAddress::from(_user2), 2000u64)); //20%

//     //make owner admin first
//     state.world.sc_call(
//         ScCallStep::new()
//             .from(OWNER_ADDR)
//             .to(GAME_SC_ADDR)
//             .call(game_sc.set_admin(ManagedAddress::from(_owner)))
//             .expect(TxExpect::ok()),
//     );

//     //send reward
//     state.send_reward(
//         1u64,
//         &mut game_sc,
//         OptionalValue::Some(winners),
//         OptionalValue::None,
//     );

//     //user1 should receive 80% of the reward
//     //reward = 3 * wager = 300 => user1's reward = 240
//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             USER1_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, "100000140"), //balance - wager + 240
//         ));

//     //user2's reward = 60
//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             USER2_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, "99999960"), //balance - wager + 60
//         ));
// }

// #[test]
// fn invalid_game_test() {
//     let mut state = GameContractState::new();
//     let mut game_sc = ContractInfo::<mvx_game_sc::Proxy<StaticApi>>::new(GAME_SC_ADDR);

//     //game settings
//     let waiting_time = 100u64; // => timestamp 102 should be out of waiting time
//     let number_of_players_min = 3u64;
//     let number_of_players_max = 5u64;
//     let wager = BigUint::from(100u64);
//     let _owner = state.owner.clone();

//     //deploy
//     state.deploy(&mut game_sc);

//     //set now = 1
//     state
//         .world
//         .set_state_step(SetStateStep::new().block_timestamp(1u64));

//     //create game
//     state.create_game(
//         waiting_time,
//         number_of_players_min,
//         number_of_players_max,
//         wager.clone(),
//         OWNER_ADDR,
//         &mut game_sc,
//         "1",
//     );

//     //user1 joins
//     state.join_game(
//         1u64,
//         USER1_ADDR,
//         wager.clone(),
//         &mut game_sc,
//         OptionalValue::None,
//     );

//     //user2 joins
//     state.join_game(1u64, USER2_ADDR, wager, &mut game_sc, OptionalValue::None);

//     //game is still invalid, min number of players not reached
//     let game_settings: SingleValue<GameSettings<StaticApi>> = game_sc
//         .game_settings(1u64)
//         .into_vm_query()
//         .expect(TxExpect::ok())
//         .execute(&mut state.world);

//     assert_eq!(game_settings.into().status, Status::Invalid);

//     //set now = 102, past waiting time
//     state
//         .world
//         .set_state_step(SetStateStep::new().block_timestamp(102u64));

//     //make owner admin first
//     state.world.sc_call(
//         ScCallStep::new()
//             .from(OWNER_ADDR)
//             .to(GAME_SC_ADDR)
//             .call(game_sc.set_admin(ManagedAddress::from(_owner)))
//             .expect(TxExpect::ok()),
//     );

//     //send reward, invalid game => players should receive back wager, creator should receive the creation fee back
//     state.send_reward(1u64, &mut game_sc, OptionalValue::None, OptionalValue::None);

//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             USER1_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, "100000000"),
//         ));

//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             USER2_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, "100000000"),
//         ));

//     state
//         .world
//         .check_state_step(CheckStateStep::new().put_account(
//             OWNER_ADDR,
//             CheckAccount::new().esdt_balance(TOKEN_ID, "100000000"),
//         ));
// }
