use adder::adder_proxy;
use imports::{
    EgldOrEsdtTokenIdentifier, MxscPath, TestAddress, TestEsdtTransfer, TestSCAddress,
    TestTokenIdentifier,
};
use multiversx_sc::{
    codec::{multi_types::MultiValueVec, top_encode_to_vec_u8_or_panic},
    types::{BigUint, MultiValueEncoded},
};
use multiversx_sc_scenario::*;
use multiversx_wegld_swap_sc::wegld_proxy;
use paymaster::paymaster_proxy;

const PAYMASTER_ADDRESS_EXPR: TestSCAddress = TestSCAddress::new("paymaster");
const RELAYER_ADDRESS_EXPR: TestAddress = TestAddress::new("relayer");
const CALLEE_SC_ADDER_ADDRESS_EXPR: TestSCAddress = TestSCAddress::new("adder");
const CALLEE_SC_WEGLD_ADDRESS_EXPR: TestSCAddress = TestSCAddress::new("wegld");
const PAYMASTER_PATH_EXPR: MxscPath = MxscPath::new("output/paymaster.mxsc.json");
const ADDER_PATH_EXPR: MxscPath = MxscPath::new("../adder/output/adder.mxsc.json");
const WEGLD_PATH_EXPR: MxscPath =
    MxscPath::new("../wegld-swap/output/multiversx-wegld-swap-sc.mxsc.json");
const CALLER_ADDRESS_EXPR: TestAddress = TestAddress::new("caller");
const CALLEE_USER_ADDRESS_EXPR: TestAddress = TestAddress::new("callee_user");
const OWNER_ADDRESS_EXPR: TestAddress = TestAddress::new("owner");
const BALANCE: u64 = 100_000_000;
const PAYMASTER_TOKEN_ID_EXPR: TestTokenIdentifier = TestTokenIdentifier::new("PAYMSTR-123456");
const WEGLD_TOKEN_ID_EXPR: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
const FEE_TOKEN_ID_EXPR: TestTokenIdentifier = TestTokenIdentifier::new("FEE-123456");
const ADDITIONAL_TOKEN_ID_EXPR: TestTokenIdentifier = TestTokenIdentifier::new("ADDIT-123456");
const FEE_AMOUNT: u64 = 20_000;
const INITIAL_ADD_VALUE: u64 = 5;
const ADDITIONAL_ADD_VALUE: u64 = 5;
const UNWRAP_ENDPOINT_NAME: &[u8] = b"unwrap";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/paymaster");
    blockchain.register_contract(PAYMASTER_PATH_EXPR, paymaster::ContractBuilder);
    blockchain.register_contract(ADDER_PATH_EXPR, adder::ContractBuilder);
    blockchain.register_contract(WEGLD_PATH_EXPR, multiversx_wegld_swap_sc::ContractBuilder);

    blockchain
}

struct PaymasterTestState {
    world: ScenarioWorld,
}

impl PaymasterTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace();
        world.account(OWNER_ADDRESS_EXPR).nonce(1);
        world
            .account(CALLER_ADDRESS_EXPR)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(PAYMASTER_TOKEN_ID_EXPR, BALANCE)
            .esdt_balance(WEGLD_TOKEN_ID_EXPR, BALANCE)
            .esdt_balance(FEE_TOKEN_ID_EXPR, BALANCE)
            .esdt_balance(ADDITIONAL_TOKEN_ID_EXPR, BALANCE);

        world
            .account(CALLEE_USER_ADDRESS_EXPR)
            .nonce(1)
            .balance(BALANCE);
        world.account(RELAYER_ADDRESS_EXPR).nonce(1).balance(0);

        Self { world }
    }

    fn deploy_paymaster_contract(&mut self) -> &mut Self {
        self.world
            .new_address(OWNER_ADDRESS_EXPR, 1, PAYMASTER_ADDRESS_EXPR);

        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .typed(paymaster_proxy::PaymasterContractProxy)
            .init()
            .code(PAYMASTER_PATH_EXPR)
            .new_address(PAYMASTER_ADDRESS_EXPR)
            .run();

        self
    }

    fn deploy_adder_contract(&mut self) -> &mut Self {
        self.world
            .new_address(OWNER_ADDRESS_EXPR, 2, CALLEE_SC_ADDER_ADDRESS_EXPR);

        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .typed(adder_proxy::AdderProxy)
            .init(INITIAL_ADD_VALUE)
            .code(ADDER_PATH_EXPR)
            .new_address(CALLEE_SC_ADDER_ADDRESS_EXPR)
            .run();

        self
    }

    fn deploy_wegld_contract(&mut self) -> &mut Self {
        self.world
            .new_address(OWNER_ADDRESS_EXPR, 3, CALLEE_SC_WEGLD_ADDRESS_EXPR);

        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .typed(wegld_proxy::EgldEsdtSwapProxy)
            .init(WEGLD_TOKEN_ID_EXPR)
            .code(WEGLD_PATH_EXPR)
            .new_address(CALLEE_SC_WEGLD_ADDRESS_EXPR)
            .run();

        self
    }

    fn check_esdt_balance(
        &mut self,
        address: TestAddress,
        token: TestTokenIdentifier,
        balance: u64,
    ) -> &mut Self {
        self.world
            .check_account(address)
            .esdt_balance(token, balance);

        self
    }

    fn check_egld_balance(&mut self, address: TestAddress, balance: u64) -> &mut Self {
        self.world.check_account(address).balance(balance);

        self
    }
}

#[test]
fn test_deploy_paymasters() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();
    state.deploy_wegld_contract();
}

#[test]
fn test_forward_call_no_fee_payment() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();

    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_USER_ADDRESS_EXPR,
            0u64,
            b"add",
            MultiValueVec::<Vec<u8>>::new(),
        )
        .with_result(ExpectError(4, "There is no fee for payment!"))
        .run();
}

#[test]
fn test_forward_call_user() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();

    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_USER_ADDRESS_EXPR,
            0u64,
            b"add",
            MultiValueVec::<Vec<u8>>::new(),
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(FEE_TOKEN_ID_EXPR),
            0u64,
            &BigUint::from(FEE_AMOUNT),
        )
        .run();

    state
        .world
        .check_account(RELAYER_ADDRESS_EXPR)
        .esdt_balance(FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
}

#[test]
fn test_forward_call_sc_adder() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, PAYMASTER_TOKEN_ID_EXPR, BALANCE);

    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_SC_ADDER_ADDRESS_EXPR,
            0u64,
            b"add",
            MultiValueVec::from([top_encode_to_vec_u8_or_panic(&ADDITIONAL_ADD_VALUE)]),
        )
        .esdt(TestEsdtTransfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT))
        .run();

    let expected_adder_sum = INITIAL_ADD_VALUE + ADDITIONAL_ADD_VALUE;

    state
        .world
        .query()
        .to(CALLEE_SC_ADDER_ADDRESS_EXPR)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .with_result(ExpectValue(expected_adder_sum))
        .run();
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
}

#[test]
fn test_forward_call_sc_adder_with_relayer_address() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, PAYMASTER_TOKEN_ID_EXPR, BALANCE);

    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_SC_ADDER_ADDRESS_EXPR,
            0u64,
            b"add",
            MultiValueVec::from([top_encode_to_vec_u8_or_panic(&ADDITIONAL_ADD_VALUE)]),
        )
        .esdt(TestEsdtTransfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT))
        .run();

    let expected_adder_sum = INITIAL_ADD_VALUE + ADDITIONAL_ADD_VALUE;
    state
        .world
        .query()
        .to(CALLEE_SC_ADDER_ADDRESS_EXPR)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .with_result(ExpectValue(expected_adder_sum))
        .run();

    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
}

#[test]
fn test_forward_call_wegld() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, WEGLD_TOKEN_ID_EXPR, BALANCE);

    let payments = vec![
        TestEsdtTransfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT),
        TestEsdtTransfer(WEGLD_TOKEN_ID_EXPR, 0, FEE_AMOUNT),
    ];

    // Call fails because unwrap amount is 0
    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_SC_WEGLD_ADDRESS_EXPR,
            0u64,
            UNWRAP_ENDPOINT_NAME,
            MultiValueEncoded::new(),
        )
        .multi_esdt(payments)
        .run();

    // Fee is kept by the relayer
    let new_fee_amount: u64 = 99_980_000;
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, new_fee_amount);

    // Caller has the original balance
    state.check_egld_balance(CALLER_ADDRESS_EXPR, BALANCE);
}

#[test]
fn test_forward_call_fails_wegld_0_amount() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, WEGLD_TOKEN_ID_EXPR, BALANCE);

    let failling_amount = 0u64;

    let payments = vec![
        TestEsdtTransfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT),
        TestEsdtTransfer(WEGLD_TOKEN_ID_EXPR, 0, failling_amount),
    ];

    // Call fails because unwrap amount is 0
    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_SC_WEGLD_ADDRESS_EXPR,
            0u64,
            UNWRAP_ENDPOINT_NAME,
            MultiValueEncoded::new(),
        )
        .multi_esdt(payments)
        .run();

    // Fee is kept by the relayer
    let new_fee_amount: u64 = 99_980_000;
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, new_fee_amount);

    // Caller has the original balance
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, WEGLD_TOKEN_ID_EXPR, BALANCE);
}

#[test]
fn test_forward_call_fails_check_amounts() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_wegld_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, WEGLD_TOKEN_ID_EXPR, BALANCE);

    let mut payments = Vec::new();
    payments.push(TestEsdtTransfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT));

    let sent_amount = 1_000u64;
    payments.push(TestEsdtTransfer(WEGLD_TOKEN_ID_EXPR, 0, sent_amount));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS_EXPR)
        .to(CALLEE_SC_WEGLD_ADDRESS_EXPR)
        .typed(wegld_proxy::EgldEsdtSwapProxy)
        .pause_endpoint()
        .run();

    // Call fails because wrong WEGLD token provided
    state
        .world
        .tx()
        .from(CALLER_ADDRESS_EXPR)
        .to(PAYMASTER_ADDRESS_EXPR)
        .typed(paymaster_proxy::PaymasterContractProxy)
        .forward_execution(
            RELAYER_ADDRESS_EXPR,
            CALLEE_SC_WEGLD_ADDRESS_EXPR,
            100u64,
            UNWRAP_ENDPOINT_NAME,
            MultiValueEncoded::new(),
        )
        .multi_esdt(payments)
        .run();

    // Fee is kept by the relayer
    let new_fee_amount: u64 = 99_980_000;
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, new_fee_amount);

    // Caller has the original balance
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, WEGLD_TOKEN_ID_EXPR, BALANCE);
}
