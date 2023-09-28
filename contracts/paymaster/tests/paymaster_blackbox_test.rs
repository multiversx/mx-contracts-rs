use multiversx_sc::types::Address;
use multiversx_sc_scenario::{api::StaticApi, *, scenario_model::{ScDeployStep, AddressValue, Account, SetStateStep}};
use paymaster::ProxyTrait;

const PAYMASTER_ADDRESS_EXPR: &str = "sc:paymaster";
const RELAYER_ADDRESS_EXPR: &str = "sc:relayer";
const CALLEE_SC_ADDRESS_EXPR: &str = "sc:callee_sc";
const PAYMASTER_PATH_EXPR: &str = "file:output/paymaster.wasm";
const CALLER_ADDRESS_EXPR: &str = "address:caller";
const CALLEE_USER_ADDRESS_EXPR: &str = "address:callee_user";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const BALANCE: &str = "100,000,000";

type PaymasterContract = ContractInfo<paymaster::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/examples/empty");

    blockchain.register_contract("file:output/paymaster.wasm", paymaster::ContractBuilder);
    blockchain
}

struct PaymasterTestState {
    world: ScenarioWorld,
    caller_address: Address,
    callee_user_address: Address,
    paymaster_contract: PaymasterContract,
    relayer_address: Address,
    callee_sc_address: Address,
}

impl PaymasterTestState {
    fn new() -> Self {
        let mut world = world();
        world.set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1))
                .put_account(
                    CALLER_ADDRESS_EXPR,
                    Account::new().nonce(1).balance(BALANCE),
                )
                .put_account(CALLEE_USER_ADDRESS_EXPR, Account::new().nonce(1).balance(BALANCE))
                .new_address(OWNER_ADDRESS_EXPR, 1, PAYMASTER_ADDRESS_EXPR)
                .new_address(OWNER_ADDRESS_EXPR, 1, RELAYER_ADDRESS_EXPR)
                .new_address(OWNER_ADDRESS_EXPR, 1, CALLEE_SC_ADDRESS_EXPR),
        );

        let paymaster_contract = PaymasterContract::new(PAYMASTER_ADDRESS_EXPR);

        let caller_address = AddressValue::from(CALLER_ADDRESS_EXPR).to_address();
        let callee_user_address = AddressValue::from(CALLEE_USER_ADDRESS_EXPR).to_address();

        let relayer_address = AddressValue::from(RELAYER_ADDRESS_EXPR).to_address();
        let callee_sc_address = AddressValue::from(CALLEE_SC_ADDRESS_EXPR).to_address();

        Self {
            world,
            caller_address,
            callee_user_address,
            paymaster_contract,
            relayer_address,
            callee_sc_address,
        }
    }

    fn deploy_paymaster_contract(&mut self) -> &mut Self {
        let paymaster_code = self.world.code_expression(PAYMASTER_PATH_EXPR);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(paymaster_code)
                .call(self.paymaster_contract.init()),
        );
        self
    }
}

#[test]
fn test_deploy_paymaster() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
}
