use std::fmt::Result;

use multiversx_sc_scenario::{imports::*, ScenarioWorld};
use potlock::{potlock_proxy, potlock_storage::PotlockId};

const POTLOCK_ADDRESS: TestSCAddress = TestSCAddress::new("potlock");
const POTLOCK_CODE_PATH: MxscPath = MxscPath::new("output/potlock.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const ADMIN_ADDRESS: TestAddress = TestAddress::new("admin");
const POT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("pot_proposer");
const PROJECT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("project_proposer");
const POT_DONOR_ADDRESS: TestAddress = TestAddress::new("pot_donor");
const PROJECT_DONOR_ADDRESS: TestAddress = TestAddress::new("project_donor");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("POT-123456");
const POT_FEE_CREATION: u64 = 1_000; // 1 week in seconds

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(POTLOCK_CODE_PATH, potlock::ContractBuilder);
    blockchain
}

struct PotlockTestState {
    world: ScenarioWorld,
}

impl PotlockTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .account(ADMIN_ADDRESS)
            .nonce(1)
            .account(POT_PROPOSER_ADDRESS)
            .nonce(1)
            .esdt_balance(TOKEN_ID, 1_000)
            .account(PROJECT_PROPOSER_ADDRESS)
            .nonce(1)
            .account(POT_DONOR_ADDRESS)
            .nonce(1);

        Self { world }
    }

    fn deploy_potlock_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .init(ADMIN_ADDRESS)
            .code(POTLOCK_CODE_PATH)
            .new_address(POTLOCK_ADDRESS)
            .run();
        self
    }

    fn change_fee_for_pots(&mut self, fee_amount: u64) {
        self.world
            .tx()
            .from(ADMIN_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .change_fee_for_pots(TokenIdentifier::from(TOKEN_ID), BigUint::from(fee_amount))
            .run();
    }

    fn add_pot(&mut self, name: &str, description: &str) {
        self.world
            .tx()
            .from(POT_PROPOSER_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .add_pot(name, description)
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
                0u64,
                &multiversx_sc::proxy_imports::BigUint::from(POT_FEE_CREATION),
            )
            .run();
    }

    fn accept_pot(&mut self, potlock_id: PotlockId) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .accept_pot(potlock_id)
            .run();
    }

    fn remove_pot(&mut self, potlock_id: PotlockId) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .remove_pot(potlock_id)
            .run();
    }

    fn apply_for_pot(&mut self, potlock_id: PotlockId, project_name: &str, description: &str) {
        self.world
            .tx()
            .from(PROJECT_PROPOSER_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .apply_for_pot(potlock_id, project_name, description)
            .run();
    }

    ////////// Checks //////////
    fn check_esdt_balance(&mut self, address: TestAddress, balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, balance);
    }

    fn check_sc_esdt_balance(&mut self, address: TestSCAddress, balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, balance);
    }

    fn check_potlock_id(&mut self, potlock_id: PotlockId) {
        let potlocks = self
            .world
            .query()
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .potlocks()
            .returns(ReturnsResult)
            .run();
    }
}

#[test]
fn test_deploy_and_config() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);
}

#[test]
fn test_add_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, 0);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_accept_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;

    state.accept_pot(potlock_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, 0);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_remove_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;

    state.remove_pot(potlock_id);

    // Funds were returned to user
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, 0);
}

#[test]
fn test_apply_for_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;

    state.apply_for_pot(potlock_id, "Project name", "Project description");

    // Funds were returned to user
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, 0);
}

///////////// Negative tests //////////////

#[test]
fn test_fail_add_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state
        .world
        .tx()
        .from(POT_PROPOSER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .add_pot("name", "description")
        .with_result(ExpectError(4, "incorrect number of ESDT transfers"))
        .run();

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, 0);
}

#[test]
fn test_fail_accept_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");

    let potlock_id = 1usize;
    state
        .world
        .tx()
        .from(POT_PROPOSER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .accept_pot(potlock_id)
        .with_result(ExpectError(4, "Endpoint can only be called by admins"))
        .run();

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, 0);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_fail_remove_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");

    let potlock_id = 1usize;
    state
        .world
        .tx()
        .from(POT_PROPOSER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .remove_pot(potlock_id)
        .with_result(ExpectError(4, "Endpoint can only be called by admins"))
        .run();

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, 0);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}
