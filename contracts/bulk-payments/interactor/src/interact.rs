#![allow(non_snake_case)]

pub mod config;
mod proxy;

use config::Config;
use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

pub async fn bulk_payments_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let config = Config::new();
    let mut interact = ContractInteract::new(config).await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "joinParty" => interact.join_party().await,
        "bulksend" => interact.bulksend().await,
        "getNativeToken" => interact.native_token().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    contract_address: Option<Bech32Address>
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

pub struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    bob: Address,
    ivan: Address,
    heidi: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("bulk-payments");
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;
        let bob = interactor.register_wallet(test_wallets::bob()).await;
        let ivan = interactor.register_wallet(test_wallets::ivan()).await;
        let heidi = interactor.register_wallet(test_wallets::heidi()).await;



        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/bulk-payments.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            bob,
            ivan,
            heidi,
            contract_code,
            state: State::load_state()
        }
    }

    pub async fn deploy(&mut self) {
        let native_token = TokenIdentifier::from_esdt_bytes(b"WEGLD-a28c59");

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::BulkPaymentsProxy)
            .init(native_token)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    pub async fn upgrade(&mut self) {
        let native_token = TokenIdentifier::from_esdt_bytes(b"WEGLD-a28c59");

        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::BulkPaymentsProxy)
            .upgrade(native_token)
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn join_party(&mut self) {
        // let user = Bech32Address::from_bech32_string("erd1czjv9ku90wz8vhjw2zyqst9avnsr2vvkv2et8gpht3fm8qlk4t9saphamf".to_string());

        let response = self
            .interactor
            .tx()
            .from(&self.heidi)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::BulkPaymentsProxy)
            .join_party()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");

        let response = self
        .interactor
        .tx()
        .from(&self.ivan)
        .to(self.state.current_address())
        .gas(30_000_000u64)
        .typed(proxy::BulkPaymentsProxy)
        .join_party()
        .returns(ReturnsResultUnmanaged)
        .run()
        .await;

    println!("Result: {response:?}");

    }

    pub async fn bulksend(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(b"WEGLD-a28c59");
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(10000000000000000u128);

        let payment_amount = BigUint::<StaticApi>::from(100000000000000000u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::BulkPaymentsProxy)
            .bulksend(payment_amount)
            .payment((token_id, token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn native_token(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::BulkPaymentsProxy)
            .native_token()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

}
