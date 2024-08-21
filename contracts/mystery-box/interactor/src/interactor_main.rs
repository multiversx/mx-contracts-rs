#![allow(non_snake_case)]
#[warn(dead_code)]
mod proxy;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
//use mystery_box::__wasm__endpoints__::set_roles;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

//use mystery_box::config::RewardType;
use crate::proxy::RewardType;

const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_IDENTIFIER: &str = "TTO-281def";
const INVALID_TOKEN_ID: &str = "xyz";

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    // let cmd = args.next().expect("at least one argument required");
    // let mut interact = ContractInteract::new().await;
    // match cmd.as_str() {
    //     "deploy" => interact.deploy().await,
    //     "setupMysteryBox" => interact.setup_mystery_box().await,
    //     "updateMysteryBoxUris" => interact.update_mystery_box_uris().await,
    //     "createMysteryBox" => interact.create_mystery_box().await,
    //     "openMysteryBox" => interact.open_mystery_box().await,
    //     "getMysteryBoxTokenIdentifier" => interact.mystery_box_token_id().await,
    //     "getGlobalCooldownEpoch" => interact.global_cooldown_epoch().await,
    //   "getWinningRates" => interact.winning_rates().await,
    //     "getMysteryBoxUris" => interact.mystery_box_uris().await,
    //     "isAdmin" => interact.is_admin().await,
    //    // "addAdmin" => interact.add_admin().await,
    //     "removeAdmin" => interact.remove_admin().await,
    //    "getAdmins" => interact.admins().await,
    //     _ => panic!("unknown command: {}", &cmd),
    // }
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
    user1_address: Address,
    user2_address: Address,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());
        let user1_address = interactor.register_wallet(test_wallets::dan());
        let user2_address = interactor.register_wallet(test_wallets::frank());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/mystery-box.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            user1_address,
            user2_address,
            contract_code,
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self) {
        let mystery_box_token_id = TokenIdentifier::from_esdt_bytes(TOKEN_IDENTIFIER);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .init(mystery_box_token_id)
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

    async fn deploy_fail(&mut self, expected_result: ExpectError<'_>) {
        const TOKEN_IDENTIFIER: &str = "ax";
        let mystery_box_token_id = TokenIdentifier::from_esdt_bytes(TOKEN_IDENTIFIER);
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .init(mystery_box_token_id)
            .code(&self.contract_code)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;
        //    let new_address_bech32 = bech32::encode(&new_address);
        //    self.state
        //    .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));
        //    println!("new address: {new_address_bech32}");
    }

    async fn setup_mystery_box1(&mut self) {
        let mut winning_rates_list = MultiValueVec::<
            MultiValue6<
                RewardType,
                EgldOrEsdtTokenIdentifier<StaticApi>,
                BigUint<StaticApi>,
                ManagedBuffer<StaticApi>,
                u64,
                u64,
            >,
        >::new();

        let mut reward1 = (
            RewardType::FixedValue,
            EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDENTIFIER)),
            BigUint::from(5u128),
            managed_buffer!(b"FixedValue"),
            3_000,
            1,
        )
            .into();
        winning_rates_list.push(reward1);

        let mut reward2 = (
            RewardType::CustomReward,
            EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDENTIFIER)),
            BigUint::from(5u128),
            managed_buffer!(b"CustomText"),
            7_000,
            2,
        )
            .into();
        winning_rates_list.push(reward2);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address) // Verifică dacă această adresă are permisiuni de admin
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .setup_mystery_box(winning_rates_list)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn setup_mystery_box_fail_percentage(&mut self, expected_result: ExpectError<'_>) {
        let mut winning_rates_list = MultiValueVec::<
            MultiValue6<
                RewardType,
                EgldOrEsdtTokenIdentifier<StaticApi>,
                BigUint<StaticApi>,
                ManagedBuffer<StaticApi>,
                u64,
                u64,
            >,
        >::new();

        let mut reward1 = (
            RewardType::FixedValue,
            EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDENTIFIER)),
            BigUint::from(5u128),
            managed_buffer!(b"FixedValue"),
            2_000,
            1,
        )
            .into();
        winning_rates_list.push(reward1);

        let mut reward2 = (
            RewardType::CustomReward,
            EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDENTIFIER)),
            BigUint::from(5u128),
            managed_buffer!(b"CustomText"),
            7_000,
            2,
        )
            .into();
        winning_rates_list.push(reward2);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address) // Verifică dacă această adresă are permisiuni de admin
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .setup_mystery_box(winning_rates_list)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn setup_mystery_box_one_reward(&mut self) {
        let mut winning_rates_list = MultiValueVec::<
            MultiValue6<
                RewardType,
                EgldOrEsdtTokenIdentifier<StaticApi>,
                BigUint<StaticApi>,
                ManagedBuffer<StaticApi>,
                u64,
                u64,
            >,
        >::new();

        let mut reward1 = (
            RewardType::ExperiencePoints,
            EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDENTIFIER)),
            BigUint::from(5u128),
            managed_buffer!(b"ExperiencePoints"),
            10_000,
            1,
        )
            .into();
        winning_rates_list.push(reward1);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address) // Verifică dacă această adresă are permisiuni de admin
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .setup_mystery_box(winning_rates_list)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn setup_mystery_box(&mut self) {
        let winning_rates_list = MultiValueVec::<
            MultiValue6<
                RewardType,
                EgldOrEsdtTokenIdentifier<StaticApi>,
                BigUint<StaticApi>,
                ManagedBuffer<StaticApi>,
                u64,
                u64,
            >,
        >::new();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .setup_mystery_box(winning_rates_list)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn update_mystery_box_uris(&mut self) {
        let uris = MultiValueVec::from(vec![ManagedBuffer::new_from_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .update_mystery_box_uris(uris)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_mystery_box(&mut self, amount: BigUint<StaticApi>) {
        //  let amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .create_mystery_box(amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_mystery_box_fail(
        &mut self,
        amount: BigUint<StaticApi>,
        expected_result: ExpectError<'_>,
    ) {
        //  let amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .create_mystery_box(amount)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    /*
     async fn set_roles(&mut self) {
           let response = self
               .interactor
               .tx()
               .from(&self.wallet_address)
               .to(self.state.current_address())
               .gas(60_000_000u64)
               .typed(proxy::MysteryBoxProxy)
               .set_roles()
               .returns(ReturnsResultUnmanaged)
               .prepare_async()
               .run()
               .await;


           println!("Result: {response:?}");
       }

    */

    async fn open_mystery_box(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .open_mystery_box()
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn mystery_box_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .mystery_box_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn global_cooldown_epoch(&mut self, reward: RewardType) -> BigUint<StaticApi> {
        // let reward = RewardType::None;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .global_cooldown_epoch(reward)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
        BigUint::from(result_value)
    }

    async fn winning_rates(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .winning_rates()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {:#?}", result_value);
    }

    async fn mystery_box_uris(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .mystery_box_uris()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn is_admin(&mut self, adresa: Bech32Address) {
        //   let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .is_admin(adresa)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_admin(&mut self, new_admin: Bech32Address) {
        //  let address = bech32::decode("");

        //  let address = &self.user_address;
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .add_admin(new_admin)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Add admin: {response:?}");
    }

    async fn remove_admin(&mut self, admin: Bech32Address) {
        //   let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MysteryBoxProxy)
            .remove_admin(admin)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Remove admin: {response:?}");
    }

    async fn admins(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MysteryBoxProxy)
            .admins()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Admins: {result_value:?}");
    }

    //ERROR

    async fn set_roles(&mut self) {
        let address = self.state.current_address();
        let managed_address = ManagedAddress::from_address(&address.to_address());

        let mystery_box_token_id = TokenIdentifier::from_esdt_bytes(TOKEN_IDENTIFIER);
        let roles = vec![EsdtLocalRole::Mint];

        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(ESDTSystemSCAddress.to_managed_address())
            .typed(ESDTSystemSCProxy)
            .set_special_roles(&managed_address, &mystery_box_token_id, roles.into_iter())
            .prepare_async()
            .run()
            .await;
    }
}

//TESTE

#[tokio::test]
async fn test_deploy() {
    let mut interact = ContractInteract::new().await;

    interact.deploy().await;
}

#[tokio::test]
async fn test_deploy_fail() {
    let mut interact = ContractInteract::new().await;
    interact
        .deploy_fail(ExpectError(4, "Invalid token ID"))
        .await;
}

#[tokio::test]
async fn test_add_admin() {
    let mut interact = ContractInteract::new().await;

    let admin_nou: Bech32Address = interact.user1_address.clone().into();

    interact.add_admin(admin_nou).await;
}

#[tokio::test]
async fn test_remove_admin() {
    let mut interact = ContractInteract::new().await;

    let admin_sters: Bech32Address = interact.user1_address.clone().into();

    interact.remove_admin(admin_sters).await;
}

#[tokio::test]
async fn test_get_admins() {
    let mut interact = ContractInteract::new().await;
    interact.admins().await;
}

#[tokio::test]
async fn test_is_admin() {
    let mut interact = ContractInteract::new().await;

    let adresa: Bech32Address = interact.wallet_address.clone().into();
    interact.is_admin(adresa).await;
}

#[tokio::test]
async fn test_create_mystery_box_without_setup() {
    let mut interact = ContractInteract::new().await;
    let amount = BigUint::<StaticApi>::from(1u128);
    interact
        .create_mystery_box_fail(
            amount,
            ExpectError(4, "The Mystery Box must be set up first"),
        )
        .await;
}

#[tokio::test]
async fn test_setup_mysterybox() {
    let mut interact = ContractInteract::new().await;
    interact.setup_mystery_box1().await;
}

#[tokio::test]
async fn test_setup_mysterybox_fail_percentage() {
    let mut interact = ContractInteract::new().await;
    interact
        .setup_mystery_box_fail_percentage(ExpectError(4, "The total percentage must be 100%"))
        .await;
}

#[tokio::test]
async fn test_winning_rates() {
    let mut interact = ContractInteract::new().await;
    interact.winning_rates().await;
}

#[tokio::test]
async fn test_global_cooldown_epoch() {
    let mut interact = ContractInteract::new().await;
    let reward1 = RewardType::FixedValue;
    let reward2 = RewardType::CustomReward;
    interact.global_cooldown_epoch(reward1).await;
}

#[tokio::test]
async fn test_mystery_box_uris() {
    let mut interact = ContractInteract::new().await;
    interact.mystery_box_uris().await;
}

#[tokio::test]
async fn test_mystery_box_token_id() {
    let mut interact = ContractInteract::new().await;
    interact.mystery_box_token_id().await;
}

//ERROR

#[tokio::test]
async fn test_create_mystery_box() {
    let mut interact = ContractInteract::new().await;
    interact.deploy().await;
    let amount = BigUint::<StaticApi>::from(0u128);
    interact.set_roles();
    interact.setup_mystery_box1().await;
    interact.create_mystery_box(amount).await;
}
