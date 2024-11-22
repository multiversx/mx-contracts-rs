#![allow(non_snake_case)]

mod paymaster_config;
mod proxy;

use multiversx_sc_snippets::imports::*;
use paymaster_config::Config;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

const ONE_UNIT: u64 = 1_000_000_000_000_000_000;
const ONE_MILLION: u64 = 1_000_000;

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-a28c59";
pub static MEX_TOKEN_ID: &[u8] = b"MEX-a659d0";
pub static ONE_TOKEN_ID: &[u8] = b"ONE-83a7c0";
pub static USDC_TOKEN_ID: &[u8] = b"USDC-350c4e";
pub static UTK_TOKEN_ID: &[u8] = b"UTK-14d57d";

pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "forwardExecution" => interact.forward_execution().await,
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
    config: Config,
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let config = Config::load_config();
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;

        interactor.set_current_dir_from_workspace("contracts/paymaster/interactor");
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/paymaster.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            config: Config::load_config(),
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::PaymasterContractProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    async fn forward_execution(&mut self) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(ONE_UNIT) * ONE_MILLION;

        let relayer_addr = &self.config.relayer_addr;

        let dest = &self.config.egld_mex_pair_address;
        let endpoint_name = ManagedBuffer::new_from_bytes(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME);
        let endpoint_args = MultiValueVec::from(vec![
            ManagedBuffer::new_from_bytes(WEGLD_TOKEN_ID),
            ManagedBuffer::new_from_bytes(b"1"),
        ]);

        let mut payments = ManagedVec::<StaticApi, EsdtTokenPayment<StaticApi>>::new();
        payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(WEGLD_TOKEN_ID),
            token_nonce,
            BigUint::<StaticApi>::from(ONE_UNIT / 100), // 0.01 WEGLD
        ));
        payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(MEX_TOKEN_ID),
            token_nonce,
            token_amount, // 1_000_000 MEX
        ));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::PaymasterContractProxy)
            .forward_execution(
                relayer_addr,
                dest,
                1_000_000u64,
                endpoint_name,
                endpoint_args,
            )
            .payment(payments)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }
}
