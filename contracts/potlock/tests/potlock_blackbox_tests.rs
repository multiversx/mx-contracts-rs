use multiversx_sc_scenario::{imports::*, ScenarioWorld};

const POTLOCK_ADDRESS: TestSCAddress = TestSCAddress::new("potlock");
const POTLOCK_CODE_PATH: MxscPath = MxscPath::new("output/potlock.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const POT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("pot_proposer");
const PROJECT_PROPOSER_ADDRESS: TestAddress = TestAddress::new("project_proposer");
const POT_DONOR_ADDRESS: TestAddress = TestAddress::new("pot_donor");
const PROJECT_DONOR_ADDRESS: TestAddress = TestAddress::new("project_donor");

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
            .account(POT_PROPOSER_ADDRESS)
            .nonce(1)
            .account(PROJECT_PROPOSER_ADDRESS)
            .nonce(1)
            .account(POT_DONOR_ADDRESS)
            .nonce(1);

        Self { world }
    }
}
