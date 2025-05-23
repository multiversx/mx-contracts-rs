use boost_claim::{config::ConfigModule, BoostClaimContract};
use imports::{MxscPath, TestAddress, TestSCAddress};
use multiversx_sc::types::{ManagedAddress, ManagedBuffer, MultiValueEncoded};
use multiversx_sc_modules::only_admin::OnlyAdminModule;
use multiversx_sc_scenario::*;

const BOOST_CLAIM_PATH: MxscPath = MxscPath::new("mxsc:output/boost-claim.mxsc.json");
const TOKEN_NONCE: u64 = 1;
const USER: TestAddress = TestAddress::new("user");
const OWNER: TestAddress = TestAddress::new("owner");
const ADMIN: TestAddress = TestAddress::new("admin");
const SC_ADDR: TestSCAddress = TestSCAddress::new("boost-claim");
const INITIAL_TIMESTAMP: u64 = 1000;
const TIME_DIFFERENCE: u64 = 3600;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/boost-claim");
    blockchain.register_contract(BOOST_CLAIM_PATH, boost_claim::ContractBuilder);

    blockchain
}

fn setup(world: &mut ScenarioWorld) {
    world.account(OWNER).nonce(1);
    world.account(USER).nonce(1);
    world.account(ADMIN).nonce(1);
    world
        .account(SC_ADDR)
        .nonce(1)
        .code(BOOST_CLAIM_PATH)
        .owner(OWNER);
    world.new_address(OWNER, TOKEN_NONCE, SC_ADDR);
    world
        .tx()
        .from(OWNER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = ManagedAddress::from(ADMIN.to_address());

            sc.add_admin(managed_address);
        });
    world.current_block().block_timestamp(INITIAL_TIMESTAMP);
    world
        .tx()
        .from(ADMIN)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.set_difference_between_claims(TIME_DIFFERENCE);
        });
    world
        .tx()
        .from(ADMIN)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let mut prizes = MultiValueEncoded::new();
            prizes.push(ManagedBuffer::from(b"PRIZE1"));
            prizes.push(ManagedBuffer::from(b"PRIZE2"));
            prizes.push(ManagedBuffer::from(b"PRIZE3"));
            prizes.push(ManagedBuffer::from(b"PRIZE4"));
            sc.set_levels_prizes(prizes);
        });
}

#[test]
fn setup_contract_test() {
    let mut world = world();
    setup(&mut world);
}

#[test]
fn claim_test() {
    let mut world = world();
    setup(&mut world);

    world
        .query()
        .to(SC_ADDR)
        .with_result(ExpectError(4, "storage decode error (key: addressBoostInfouser____________________________): input too short"))
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 1);
            assert_eq!(address_info.last_claim_timestamp, INITIAL_TIMESTAMP);
            assert_eq!(address_info.total_cycles_completed, 0);
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 1);
            assert_eq!(address_info.last_claim_timestamp, INITIAL_TIMESTAMP);
            assert_eq!(address_info.total_cycles_completed, 0);
        });
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .with_result(ExpectMessage("User can't claim yet"))
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .current_block()
        .block_timestamp(INITIAL_TIMESTAMP + TIME_DIFFERENCE + 1);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 2);
            assert_eq!(
                address_info.last_claim_timestamp,
                INITIAL_TIMESTAMP + TIME_DIFFERENCE + 1
            );
            assert_eq!(address_info.total_cycles_completed, 0);
        });
    world
        .current_block()
        .block_timestamp(INITIAL_TIMESTAMP + 2 * TIME_DIFFERENCE + 2);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 3);
            assert_eq!(
                address_info.last_claim_timestamp,
                INITIAL_TIMESTAMP + 2 * TIME_DIFFERENCE + 2
            );
            assert_eq!(address_info.total_cycles_completed, 0);
        });
    world
        .current_block()
        .block_timestamp(INITIAL_TIMESTAMP + 3 * TIME_DIFFERENCE + 3);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 4);
            assert_eq!(
                address_info.last_claim_timestamp,
                INITIAL_TIMESTAMP + 3 * TIME_DIFFERENCE + 3
            );
            assert_eq!(address_info.total_cycles_completed, 0);
        });
    world
        .current_block()
        .block_timestamp(INITIAL_TIMESTAMP + 4 * TIME_DIFFERENCE + 4);
    world
        .tx()
        .from(USER)
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            sc.claim();
        });
    world
        .query()
        .to(SC_ADDR)
        .whitebox(boost_claim::contract_obj, |sc| {
            let managed_address = &ManagedAddress::from(USER.to_address());
            let address_info = sc.address_boost_info(managed_address).get();

            assert_eq!(address_info.current_level, 1);
            assert_eq!(
                address_info.last_claim_timestamp,
                INITIAL_TIMESTAMP + 4 * TIME_DIFFERENCE + 4
            );
            assert_eq!(address_info.total_cycles_completed, 1);
        });
}
