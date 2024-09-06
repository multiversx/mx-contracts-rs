#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::char::MAX;
use std::result;
use std::{
    io::{Read, Write},
    path::Path,
};

const GATEWAY: &str = sdk::gateway::TESTNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_ID: &str = "VLD-070dac";
const SECOND_TOKEN_ID: &str = "SCND-620d29";
const INVALID_TOKEN_ID: &str = "123";
const FEE_AMOUNT: u128 = 1;
const DONATION_AMOUNT: u64 = 10;
const OWNER_ADDR: &str = "erd1qyu5wthldzr8wx5c9ucg8kjagg0jfs53s8nr3zpz3hypefsdd8ssycr6th";
const SECOND_USER_ADDR: &str = "erd1spyavw0956vq68xj8y4tenjpq2wd5a9p2c6j8gsz7ztyrnpxrruqzu66jx";
const THIRD_USER_ADDR: &str = "erd1k2s324ww2g0yj38qn2ch2jwctdy8mnfxep94q9arncc6xecg3xaq6mjse8";
const MAX_PERCENTAGE: u64 = 10_000;
const BIG_ID: u32 = 1000u32;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

struct ContractInteract {
    interactor: Interactor,
    owner_address: Address,
    second_address: Address,
    third_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let owner_address = interactor.register_wallet(test_wallets::alice());
        let second_address = interactor.register_wallet(test_wallets::bob());
        let third_address = interactor.register_wallet(test_wallets::carol());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/potlock.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            owner_address,
            second_address,
            third_address,
            contract_code,
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self) {
        let admins = MultiValueVec::from(vec![bech32::decode(THIRD_USER_ADDR)]);

        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .init(admins)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.owner_address)
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn change_fee_for_pots(&mut self, caller: &Bech32Address, token_id: &str, fee: u128) {
        let token_identifier = TokenIdentifier::from_esdt_bytes(token_id);
        let fee = BigUint::<StaticApi>::from(fee);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .change_fee_for_pots(token_identifier, fee)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn change_fee_for_pots_fail(
        &mut self,
        caller: &Bech32Address,
        token_id: &str,
        fee: u128,
        expected_result: ExpectError<'_>,
    ) {
        let token_identifier = TokenIdentifier::from_esdt_bytes(token_id);
        let fee = BigUint::<StaticApi>::from(fee);

        self.interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .change_fee_for_pots(token_identifier, fee)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;
    }

    async fn accept_pot(&mut self, caller: &Bech32Address, potlock_id: u32) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .accept_pot(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn accept_pot_fail(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        expected_result: ExpectError<'_>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .accept_pot(potlock_id)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_pot(&mut self, caller: &Bech32Address, potlock_id: u32) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .remove_pot(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_pot_fail(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        expected_result: ExpectError<'_>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .remove_pot(potlock_id)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn accept_application(&mut self, caller: &Bech32Address, project_id: u32) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .accept_application(project_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn accept_application_fail(
        &mut self,
        caller: &Bech32Address,
        project_id: u32,
        expected_result: ExpectError<'_>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .accept_application(project_id)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;
    }

    async fn remove_application(&mut self) {
        let project_id = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .remove_application(project_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn reject_donation(&mut self) {
        let potlock_id = 0u32;
        let user = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .reject_donation(potlock_id, user)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn distribute_pot_to_projects(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        project_percentages: MultiValueVec<MultiValue2<u32, u64>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .distribute_pot_to_projects(potlock_id, project_percentages)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn distribute_pot_to_projects_fail(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        project_percentages: MultiValueVec<MultiValue2<u32, u64>>,
        expected_result: ExpectError<'_>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .distribute_pot_to_projects(potlock_id, project_percentages)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_pot(&mut self, caller: &Bech32Address, token_id: &str, fee: u128) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(fee);

        let name = ManagedBuffer::new_from_bytes(&b""[..]);
        let description = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .add_pot(name, description)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_pot_fail(
        &mut self,
        caller: &Bech32Address,
        token_id: &str,
        fee: u128,
        expected_result: ExpectError<'_>,
    ) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(fee);

        let name = ManagedBuffer::new_from_bytes(&b""[..]);
        let description = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .add_pot(name, description)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn apply_for_pot(&mut self, caller: &Bech32Address, potlock_id: u32) {
        let project_name = ManagedBuffer::new_from_bytes(&b""[..]);
        let description = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .apply_for_pot(potlock_id, project_name, description)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_pot(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        token_id: &str,
        amount: u128,
    ) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(amount);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .donate_to_pot(potlock_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_pot_fail(
        &mut self,
        caller: &Bech32Address,
        potlock_id: u32,
        token_id: &str,
        amount: u128,
        expected_result: ExpectError<'_>,
    ) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(amount);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .donate_to_pot(potlock_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_project(
        &mut self,
        caller: &Bech32Address,
        project_id: u32,
        token_id: &str,
        amount: u128,
    ) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(amount);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .donate_to_project(project_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_project_fail(
        &mut self,
        caller: &Bech32Address,
        project_id: u32,
        token_id: &str,
        amount: u128,
        expected_result: ExpectError<'_>,
    ) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(amount);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .donate_to_project(project_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn fee_token_identifier(&mut self) -> String {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .fee_token_identifier()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        result_value.to_string()
    }

    async fn fee_amount(&mut self) -> RustBigUint {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .fee_amount()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        result_value
    }

    async fn potlocks(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .potlocks()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn projects(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .projects()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn pot_donations(&mut self) {
        let potlock_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .pot_donations(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn project_donations(&mut self) {
        let project_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .project_donations(project_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn is_admin(&mut self) {
        let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .is_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_admin(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .add_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_admin(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(proxy::PotlockProxy)
            .remove_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn admins(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .admins()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_deploy_and_config() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_add_pot() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_accept_pot() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_remove_pot() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .remove_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_pot() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_accept_application() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .accept_application(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_project() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .accept_application(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_project(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_distribute_pot_to_projects() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .accept_application(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    let project_percentages = MultiValueVec::from(vec![MultiValue2::from((1u32, MAX_PERCENTAGE))]);

    interact
        .distribute_pot_to_projects(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
            project_percentages,
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_pot_twice_with_same_token() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;

    interact
        .donate_to_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            (DONATION_AMOUNT + 1).into(),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_multiple_change_fee_for_pots() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    let fee = interact.fee_amount().await;
    let token_id = interact.fee_token_identifier().await;

    assert_eq!(fee, FEE_AMOUNT.into());
    assert_eq!(token_id, TOKEN_ID.to_string());

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            SECOND_TOKEN_ID,
            FEE_AMOUNT + 1,
        )
        .await;

    let fee = interact.fee_amount().await;
    let token_id = interact.fee_token_identifier().await;

    assert_eq!(fee, FEE_AMOUNT.into());
    assert_eq!(token_id, TOKEN_ID.to_string());
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_change_fee_for_pots_non_admin() {
    let mut interact = ContractInteract::new().await;

    interact
        .change_fee_for_pots_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            INVALID_TOKEN_ID,
            FEE_AMOUNT,
            ExpectError(4, "Endpoint can only be called by admins"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_accept_pot_non_admin() {
    let mut interact = ContractInteract::new().await;

    interact
        .accept_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            ExpectError(4, "Endpoint can only be called by admins"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_remove_pot_non_admin() {
    let mut interact = ContractInteract::new().await;

    interact
        .remove_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            ExpectError(4, "Endpoint can only be called by admins"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_accept_application_non_admin() {
    let mut interact = ContractInteract::new().await;

    interact
        .accept_application_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            ExpectError(4, "Endpoint can only be called by admins"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_distribute_pot_to_projects_non_admin() {
    let mut interact = ContractInteract::new().await;

    let project_percentages = MultiValueVec::from(vec![MultiValue2::from((1u32, MAX_PERCENTAGE))]);

    interact
        .distribute_pot_to_projects_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            project_percentages,
            ExpectError(4, "Endpoint can only be called by admins"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_distribute_pot_to_projects_more_than_max_percent() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(THIRD_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .accept_application(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    let project_percentages =
        MultiValueVec::from(vec![MultiValue2::from((1u32, MAX_PERCENTAGE + 1))]);

    interact
        .distribute_pot_to_projects_fail(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
            project_percentages,
            ExpectError(4, "Total percentages more than 100%"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_project_with_different_token() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .accept_application(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_project(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            SECOND_TOKEN_ID,
            DONATION_AMOUNT.into(),
        )
        .await;

    interact
        .donate_to_project_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
            ExpectError(4, "Already made a payment with a different TokenID"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_project_inactive_project() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .accept_pot(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .apply_for_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
        )
        .await;

    interact
        .donate_to_project_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
            ExpectError(4, "Project is not active!"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_pot_inactive_pot() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;

    interact
        .change_fee_for_pots(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .add_pot(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT,
        )
        .await;

    interact
        .donate_to_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            1u32,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
            ExpectError(4, "Pot is not active!"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_add_pot_wrong_payment() {
    let mut interact = ContractInteract::new().await;

    interact
        .add_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            SECOND_TOKEN_ID,
            FEE_AMOUNT,
            ExpectError(4, "Wrong token identifier for creating a pot!"),
        )
        .await;

    interact
        .add_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            TOKEN_ID,
            FEE_AMOUNT + 1,
            ExpectError(4, "Wrong fee amount for creating a pot"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_accept_pot_non_existent() {
    let mut interact = ContractInteract::new().await;

    interact
        .accept_pot_fail(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            BIG_ID,
            ExpectError(4, "Potlock doesn't exist!"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_remove_pot_non_existent() {
    let mut interact = ContractInteract::new().await;

    interact
        .remove_pot_fail(
            &Bech32Address::from_bech32_string(OWNER_ADDR.to_string()),
            BIG_ID,
            ExpectError(4, "Potlock doesn't exist!"),
        )
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn test_donate_to_pot_non_existent() {
    let mut interact = ContractInteract::new().await;

    interact
        .donate_to_pot_fail(
            &Bech32Address::from_bech32_string(SECOND_USER_ADDR.to_string()),
            BIG_ID,
            TOKEN_ID,
            DONATION_AMOUNT.into(),
            ExpectError(4, "Potlock doesn't exist!"),
        )
        .await;
}
