use multiversx_sc_scenario::{imports::*, ScenarioWorld};
use potlock::potlock_storage::{PotlockId, ProjectId};
use potlock_proxy::Status;
mod potlock_proxy;

const POTLOCK_ADDRESS: TestSCAddress = TestSCAddress::new("potlock");
const POTLOCK_CODE_PATH: MxscPath = MxscPath::new("output/potlock.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const ADMIN_ADDRESS: TestAddress = TestAddress::new("admin");
const POT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("pot_proposer");
const PROJECT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("project_proposer");
const POT_DONOR_ADDRESS: TestAddress = TestAddress::new("pot_donor");
const PROJECT_DONOR_ADDRESS: TestAddress = TestAddress::new("project_donor");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("POT-123456");
const DIFFERENT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("DIFFPOT-123456");
const POT_FEE_CREATION: u64 = 1_000;
const INITIAL_BALANCE: u64 = 2_000;
const DONATION_AMOUNT: u64 = 100;
const HALF_PERCENTAGE: u64 = 5_000; // 50%
const MAX_PERCENTAGE: u64 = 10_000; // 100%

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
            .esdt_balance(TOKEN_ID, INITIAL_BALANCE)
            .account(PROJECT_PROPOSER_ADDRESS)
            .nonce(1)
            .account(POT_DONOR_ADDRESS)
            .nonce(1)
            .esdt_balance(TOKEN_ID, INITIAL_BALANCE)
            .account(PROJECT_DONOR_ADDRESS)
            .nonce(1)
            .esdt_balance(TOKEN_ID, INITIAL_BALANCE)
            .esdt_balance(DIFFERENT_TOKEN_ID, INITIAL_BALANCE);

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

    fn accept_application(&mut self, project_id: ProjectId) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .accept_application(project_id)
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

    fn donate_to_pot(&mut self, potlock_id: PotlockId) {
        self.world
            .tx()
            .from(POT_DONOR_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .donate_to_pot(potlock_id)
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
                0u64,
                &multiversx_sc::proxy_imports::BigUint::from(DONATION_AMOUNT),
            )
            .run();
    }

    fn donate_to_project(&mut self, project_id: ProjectId) {
        self.world
            .tx()
            .from(PROJECT_DONOR_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .donate_to_project(project_id)
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
                0u64,
                &multiversx_sc::proxy_imports::BigUint::from(DONATION_AMOUNT),
            )
            .run();
    }

    fn distribute_pot_to_projects(
        &mut self,
        potlock_id: PotlockId,
        percentages: MultiValueVec<MultiValue2<usize, u64>>,
    ) {
        self.world
            .tx()
            .from(ADMIN_ADDRESS)
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .distribute_pot_to_projects(potlock_id, percentages)
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

    fn check_potlock_id_is_last(&mut self, potlock_id: PotlockId) {
        let potlocks = self
            .world
            .query()
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .potlocks()
            .returns(ReturnsResult)
            .run();
        assert_eq!(potlocks.len(), potlock_id);
    }

    fn check_project_id_is_last(&mut self, project_id: PotlockId) {
        let projects = self
            .world
            .query()
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .projects()
            .returns(ReturnsResult)
            .run();
        assert_eq!(projects.len(), project_id);
    }

    fn check_project_is_accepted(&mut self, project_id: PotlockId) {
        let projects = self
            .world
            .query()
            .to(POTLOCK_ADDRESS)
            .typed(potlock_proxy::PotlockProxy)
            .projects()
            .returns(ReturnsResult)
            .run();
        for project in projects.into_iter() {
            if project.project_id == project_id {
                assert_eq!(project.status, Status::Active);
            }
        }
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

    let potlock_id = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_accept_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");

    let potlock_id = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    state.accept_pot(potlock_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
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
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, 0);
}

#[test]
fn test_apply_for_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;
    let project_id = 1usize;

    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_potlock_id_is_last(potlock_id);
    state.check_project_id_is_last(project_id);

    // Funds were returned to user
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_donate_to_pot() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    // Accept Pot
    state.accept_pot(potlock_id);

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
    state.check_esdt_balance(POT_DONOR_ADDRESS, INITIAL_BALANCE);

    state.donate_to_pot(potlock_id);

    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION + DONATION_AMOUNT);
    state.check_esdt_balance(POT_DONOR_ADDRESS, INITIAL_BALANCE - DONATION_AMOUNT);
}

#[test]
fn test_donate_to_project() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    // Accept Pot
    state.accept_pot(potlock_id);

    state.apply_for_pot(potlock_id, "Project name", "Project description");
    let project_id = 1usize;
    state.check_project_id_is_last(project_id);

    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
    state.check_esdt_balance(PROJECT_DONOR_ADDRESS, INITIAL_BALANCE);

    state.donate_to_project(project_id);

    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION + DONATION_AMOUNT);
    state.check_esdt_balance(PROJECT_DONOR_ADDRESS, INITIAL_BALANCE - DONATION_AMOUNT);
}

#[test]
fn test_accept_application() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id: usize = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    let project_id = 1usize;

    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_project_id_is_last(project_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);

    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);
}

#[test]
fn test_distribute_pot_to_projects() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    // Add Pot
    state.add_pot("Pot", "Pot Description");
    let potlock_id: usize = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    // Accept Pot
    state.accept_pot(potlock_id);

    // Add Project
    let project_id = 1usize;
    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_project_id_is_last(project_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);

    // Donate to Pot
    state.donate_to_pot(potlock_id);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION + DONATION_AMOUNT);
    state.check_esdt_balance(POT_DONOR_ADDRESS, INITIAL_BALANCE - DONATION_AMOUNT);

    // Accept project
    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);

    // Distribute Pot donations to projects
    let mut percentages = MultiValueVec::new();
    percentages.push((project_id, MAX_PERCENTAGE).into());
    state.distribute_pot_to_projects(potlock_id, percentages);

    state.check_esdt_balance(PROJECT_PROPOSER_ADDRESS, DONATION_AMOUNT);
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

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE);
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

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
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

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
}

#[test]
fn test_fail_accept_application() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;
    let project_id = 1usize;

    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_potlock_id_is_last(potlock_id);
    state.check_project_id_is_last(project_id);

    // Funds were returned to user
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);

    state
        .world
        .tx()
        .from(POT_PROPOSER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .accept_application(project_id)
        .with_result(ExpectError(4, "Endpoint can only be called by admins"))
        .run();
}

#[test]
fn test_fail_distribute_pot_to_projects() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id: usize = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    let project_id = 1usize;

    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_project_id_is_last(project_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);

    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);

    let mut percentages = MultiValueVec::new();
    percentages.push((project_id, HALF_PERCENTAGE).into());
    state
        .world
        .tx()
        .from(POT_PROPOSER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .distribute_pot_to_projects(potlock_id, percentages)
        .with_result(ExpectError(4, "Endpoint can only be called by admins"))
        .run();
}

#[test]
fn test_fail_distribute_pot_to_projects2() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    // Add Pot
    state.add_pot("Pot", "Pot Description");
    let potlock_id: usize = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    // Accept Pot
    state.accept_pot(potlock_id);

    // Add Project
    let project_id = 1usize;
    state.apply_for_pot(potlock_id, "Project name", "Project description");

    state.check_project_id_is_last(project_id);
    state.check_esdt_balance(POT_PROPOSER_ADDRESS, POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);

    // Donate to Pot
    state.donate_to_pot(potlock_id);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION + DONATION_AMOUNT);
    state.check_esdt_balance(POT_DONOR_ADDRESS, INITIAL_BALANCE - DONATION_AMOUNT);

    // Accept project
    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);

    // Distribute Pot donations to projects
    let mut percentages = MultiValueVec::new();
    percentages.push((project_id, 3* HALF_PERCENTAGE).into());
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .distribute_pot_to_projects(potlock_id, percentages)
        .with_result(ExpectError(4, "Total percentages more than 100%"))
        .run();
}

#[test]
fn test_fail_donate_to_project() {
    let mut state = PotlockTestState::new();
    state.deploy_potlock_contract();
    state.change_fee_for_pots(POT_FEE_CREATION);

    state.add_pot("Pot", "Pot Description");
    let potlock_id = 1usize;
    state.check_potlock_id_is_last(potlock_id);

    // Accept Pot
    state.accept_pot(potlock_id);

    state.apply_for_pot(potlock_id, "Project name", "Project description");
    let project_id = 1usize;
    state.check_project_id_is_last(project_id);

    state.accept_application(project_id);
    state.check_project_is_accepted(project_id);

    state.check_esdt_balance(POT_PROPOSER_ADDRESS, INITIAL_BALANCE - POT_FEE_CREATION);
    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION);
    state.check_esdt_balance(PROJECT_DONOR_ADDRESS, INITIAL_BALANCE);

    state.donate_to_project(project_id);

    state.check_sc_esdt_balance(POTLOCK_ADDRESS, POT_FEE_CREATION + DONATION_AMOUNT);
    state.check_esdt_balance(PROJECT_DONOR_ADDRESS, INITIAL_BALANCE - DONATION_AMOUNT);

    state
        .world
        .tx()
        .from(PROJECT_DONOR_ADDRESS)
        .to(POTLOCK_ADDRESS)
        .typed(potlock_proxy::PotlockProxy)
        .donate_to_project(project_id)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(DIFFERENT_TOKEN_ID),
            0u64,
            &multiversx_sc::proxy_imports::BigUint::from(DONATION_AMOUNT),
        )
        .with_result(ExpectError(
            4,
            "Already made a payment with a different TokenID",
        ))
        .run();
}
