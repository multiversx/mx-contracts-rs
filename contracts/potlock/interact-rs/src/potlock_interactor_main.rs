#![allow(non_snake_case)]

mod potlock_interactor_config;
mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_ID: &str = "WEGLD-a28c59";
const FEE_AMOUNT: u64 = 50000000000000000; // 0.5

use potlock_interactor_config::Config;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "acceptPot" => interact.accept_pot().await,
        "removePot" => interact.remove_pot().await,
        "acceptApplication" => interact.accept_application().await,
        "rejectDonation" => interact.reject_donation().await,
        "distributePotToProjects" => interact.distribute_pot_to_projects().await,
        "addPot" => interact.add_pot().await,
        "applyForPot" => interact.apply_for_pot().await,
        "donateToPot" => interact.donate_to_pot().await,
        "donateToProject" => interact.donate_to_project().await,
        "changeFeeForPots" => interact.change_fee_for_pots().await,
        "getFeeTokenIdentifier" => interact.fee_token_identifier().await,
        "getFeeAmount" => interact.fee_amount().await,
        "feePotPayments" => interact.fee_pot_proposer().await,
        "feeAmountAcceptPots" => interact.fee_amount_accepted_pots().await,
        "potDonations" => interact.pot_donations().await,
        "projectDonations" => interact.project_donations().await,
        "isAdmin" => interact.is_admin().await,
        "addAdmin" => interact.add_admin().await,
        "removeAdmin" => interact.remove_admin().await,
        "getAdmins" => interact.admins().await,
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
    wallet_address: Address,
    contract_code: BytesValue,
    state: State,
    config: Config,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/potlock.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
            config: Config::load_config(),
        }
    }

    async fn deploy(&mut self) {
        let admin = &self.config.admin;

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(proxy::PotlockProxy)
            .init(admin)
            .code(&self.contract_code)
            .gas(50_000_000)
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

    async fn accept_pot(&mut self) {
        let potlock_id = 1u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .accept_pot(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_pot(&mut self) {
        let potlock_id = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .remove_pot(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn accept_application(&mut self) {
        let project_id = 1u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .accept_application(project_id)
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
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .reject_donation(potlock_id, user)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn distribute_pot_to_projects(&mut self) {
        let potlock_id = 1u32;
        let project_percentage = MultiValueVec::from(vec![MultiValue2::from((1u32, 10_000u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .distribute_pot_to_projects(potlock_id, project_percentage)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_pot(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(TOKEN_ID);
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(FEE_AMOUNT);

        let description = ManagedBuffer::new_from_bytes(b"Pot used for testing");
        let name = ManagedBuffer::new_from_bytes(b"My Pot");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .add_pot(name, description)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn apply_for_pot(&mut self) {
        let potlock_id = 1u32;
        let project_name = ManagedBuffer::new_from_bytes(b"New Testing Project");
        let description = ManagedBuffer::new_from_bytes(b"Project used for testing");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .apply_for_pot(potlock_id, project_name, description)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_pot(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(TOKEN_ID);
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(3 * FEE_AMOUNT);

        let potlock_id = 1u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .donate_to_pot(potlock_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn donate_to_project(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(TOKEN_ID);
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(3 * FEE_AMOUNT);

        let project_id = 1u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .donate_to_project(project_id)
            .payment((TokenIdentifier::from(token_id), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn change_fee_for_pots(&mut self) {
        let token_identifier = TokenIdentifier::from_esdt_bytes(TOKEN_ID);
        let fee = BigUint::<StaticApi>::from(FEE_AMOUNT);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .change_fee_for_pots(token_identifier, fee)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn fee_token_identifier(&mut self) {
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

        println!("Result: {result_value:?}");
    }

    async fn fee_amount(&mut self) {
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

        println!("Result: {result_value:?}");
    }

    async fn fee_pot_proposer(&mut self) {
        let potlock_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .fee_pot_proposer(potlock_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn fee_amount_accepted_pots(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .fee_amount_accepted_pots()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn pot_donations(&mut self) {
        let project_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::PotlockProxy)
            .pot_donations(project_id)
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
            .from(&self.wallet_address)
            .to(self.state.current_address())
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
            .from(&self.wallet_address)
            .to(self.state.current_address())
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
