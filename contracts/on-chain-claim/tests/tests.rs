use config::ConfigModule;
use imports::{
    EsdtLocalRole, MxscPath, TestAddress, TestEsdtTransfer, TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc::types::{BigUint, EsdtTokenPayment, ManagedAddress};
use multiversx_sc_modules::only_admin::OnlyAdminModule;
use multiversx_sc_scenario::*;
use on_chain_claim::*;

const ON_CHAIN_CLAIM_PATH: MxscPath = MxscPath::new("mxsc:output/on-chain-claim.mxsc.json");
const XREPAIR_TOKEN_2: TestTokenIdentifier = TestTokenIdentifier::new("XREPAIRRR-abcdef");
const XREPAIR_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("XREPAIR-abcdef");
const TOKEN_NONCE: u64 = 1;
const USER: TestAddress = TestAddress::new("user");
const OWNER: TestAddress = TestAddress::new("owner");
const SC_ADDR: TestSCAddress = TestSCAddress::new("on-chain-claim");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/on-chain-claim");
    blockchain.register_contract(ON_CHAIN_CLAIM_PATH, on_chain_claim::ContractBuilder);

    blockchain
}

fn setup_with_one_token(world: &mut ScenarioWorld) {
    let roles: Vec<String> = vec![EsdtLocalRole::NftBurn.name().to_string()];

    world.account(OWNER).nonce(1);
    world
        .account(USER)
        .nonce(1)
        .esdt_nft_balance(XREPAIR_TOKEN, TOKEN_NONCE, 1, ());
    world
        .account(SC_ADDR)
        .nonce(1)
        .code(ON_CHAIN_CLAIM_PATH)
        .owner(OWNER)
        .esdt_roles(XREPAIR_TOKEN, roles);

    world.current_block().block_epoch(20);
    world.new_address(OWNER, TOKEN_NONCE, SC_ADDR);
}

fn setup_with_two_token(world: &mut ScenarioWorld) {
    let roles: Vec<String> = vec![EsdtLocalRole::NftBurn.name().to_string()];

    world.account(OWNER).nonce(1);
    world
        .account(USER)
        .nonce(1)
        .esdt_nft_balance(XREPAIR_TOKEN, TOKEN_NONCE, 100, ())
        .esdt_nft_balance(XREPAIR_TOKEN_2, TOKEN_NONCE, 1, ());
    world
        .account(SC_ADDR)
        .nonce(1)
        .code(ON_CHAIN_CLAIM_PATH)
        .owner(OWNER)
        .esdt_roles(XREPAIR_TOKEN, roles);

    world.current_block().block_epoch(20);
    world.new_address(OWNER, TOKEN_NONCE, SC_ADDR);
}

#[test]
fn check_token_identifier() {
    let mut world = world();
    setup_with_one_token(&mut world);

    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let payment = EsdtTokenPayment::new(
                XREPAIR_TOKEN.to_token_identifier(),
                1u64,
                BigUint::from(1u64),
            );
            sc.repair_streak_payment().set(payment);
        });
}

#[test]
fn check_before_claim() {
    let mut world = world();
    setup_with_one_token(&mut world);

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        })
}

#[test]
fn check_update_state() {
    let mut world = world();
    setup_with_one_token(&mut world);

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        });
    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = ManagedAddress::from(OWNER.to_address());

            sc.add_admin(managed_address);
        });
    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            sc.update_state(managed_address, 5u64, 21u64, 7u64, 5u64);
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 5);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.total_epochs_claimed, 7);
        });
}

#[test]
fn check_after_claim() {
    let mut world = world();
    setup_with_one_token(&mut world);

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim();
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        });

    world.current_block().block_epoch(21);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
        });

    world.current_block().block_epoch(25);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 25);
        });
}

#[test]
fn check_claim_and_repair() {
    let mut world = world();

    setup_with_two_token(&mut world);

    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let payment = EsdtTokenPayment::new(
                XREPAIR_TOKEN.to_token_identifier(),
                1u64,
                BigUint::from(1u64),
            );
            sc.repair_streak_payment().set(payment);
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .with_result(ExpectMessage("can't repair streak for address"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        });

    world.current_block().block_epoch(21);

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .with_result(ExpectMessage("can't repair streak for current epoch"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        });

    world.current_block().block_epoch(22);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN_2, TOKEN_NONCE, 1))
        .with_result(ExpectMessage("Bad payment token/amount"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 2))
        .with_result(ExpectMessage("Bad payment token/amount"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 2))
        .with_result(ExpectMessage("Bad payment token/amount"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 22);
            assert_eq!(address_info.best_streak, 3);
        });

    world.current_block().block_epoch(28);

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 9);
            assert_eq!(address_info.total_epochs_claimed, 9);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 9);
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .with_result(ExpectMessage("can't repair streak for current epoch"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });
}

#[test]
fn test_best_streak() {
    let mut world = world();

    setup_with_one_token(&mut world);

    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let payment = EsdtTokenPayment::new(
                XREPAIR_TOKEN.to_token_identifier(),
                1u64,
                BigUint::from(1u64),
            );
            sc.repair_streak_payment().set(payment);
        });
    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = ManagedAddress::from(OWNER.to_address());

            sc.add_admin(managed_address);
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let repair_streak_payment = sc.repair_streak_payment().get();
            assert_eq!(
                repair_streak_payment.token_identifier,
                XREPAIR_TOKEN.to_token_identifier()
            );
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
            assert_eq!(address_info.best_streak, 1);
        });

    world.current_block().block_epoch(21);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 2);
        });

    world.current_block().block_epoch(25);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 25);
            assert_eq!(address_info.best_streak, 2);
        });

    world.current_block().block_epoch(26);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 4);
            assert_eq!(address_info.last_epoch_claimed, 26);
            assert_eq!(address_info.best_streak, 2);
        });

    world.current_block().block_epoch(27);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 5);
            assert_eq!(address_info.last_epoch_claimed, 27);
            assert_eq!(address_info.best_streak, 3);
        });
}

#[test]
fn on_chain_claim_whitebox() {
    let mut world = world();

    setup_with_one_token(&mut world);

    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let payment = EsdtTokenPayment::new(
                XREPAIR_TOKEN.to_token_identifier(),
                1u64,
                BigUint::from(1u64),
            );
            sc.repair_streak_payment().set(payment);
        });

    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = ManagedAddress::from(OWNER.to_address());

            sc.add_admin(managed_address);
        });

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let repair_streak_payment = sc.repair_streak_payment().get();
            assert_eq!(
                repair_streak_payment.token_identifier,
                XREPAIR_TOKEN.to_token_identifier()
            );
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());

    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);

            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        });

    world.current_block().block_epoch(21);
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        });

    world.current_block().block_epoch(22);
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(can_be_repaired);
        });

    world.current_block().block_epoch(26);
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(can_be_repaired);
        });

    world.current_block().block_epoch(27);
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        });

    world.current_block().block_epoch(28);
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .with_result(ExpectMessage("can't repair streak for current epoch"))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });

    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| sc.claim());
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 1);
        });
    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            sc.update_state(managed_address, 5u64, 21u64, 7u64, 5u64);
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 5);
            assert_eq!(address_info.total_epochs_claimed, 7);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 5);
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .esdt(TestEsdtTransfer(XREPAIR_TOKEN, TOKEN_NONCE, 1))
        .whitebox(on_chain_claim::contract_obj, |sc| {
            sc.claim_and_repair();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(on_chain_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 12);
            assert_eq!(address_info.total_epochs_claimed, 14);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 12);
        });
}
