use multiversx_sc::{
    codec::{multi_types::MultiValueVec, top_encode_to_vec_u8_or_panic},
    storage::mappers::SingleValue,
    types::{Address, BigUint},
};
use multiversx_sc_scenario::{
    api::StaticApi,
    scenario_model::{
        Account, AddressValue, CheckAccount, CheckStateStep, ScCallStep, ScDeployStep, ScQueryStep,
        SetStateStep, TxExpect,
    },
    *,
};

use adder::ProxyTrait as _;
use paymaster::ProxyTrait as _;

const PAYMASTER_ADDRESS_EXPR: &str = "sc:paymaster";
const RELAYER_ADDRESS_EXPR: &str = "address:relayer";
const CALLEE_SC_ADDER_ADDRESS_EXPR: &str = "sc:adder";
const PAYMASTER_PATH_EXPR: &str = "file:output/paymaster.wasm";
const ADDER_PATH_EXPR: &str = "file:tests/test-contracts/adder.wasm";
const CALLER_ADDRESS_EXPR: &str = "address:caller";
const CALLEE_USER_ADDRESS_EXPR: &str = "address:callee_user";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const BALANCE: &str = "100,000,000";
const PAYMASTER_TOKEN_ID_EXPR: &str = "str:PAYMSTR-123456";
const FEE_TOKEN_ID_EXPR: &str = "str:FEE-123456";
const ADDITIONAL_TOKEN_ID_EXPR: &str = "str:ADDIT-123456";
const FEE_AMOUNT: &str = "20,000";
const INITIAL_ADD_VALUE: u64 = 5;
const ADDITIONAL_ADD_VALUE: u64 = 5;



type PaymasterContract = ContractInfo<paymaster::Proxy<StaticApi>>;
type AdderContract = ContractInfo<adder::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/paymaster");

    blockchain.register_contract(PAYMASTER_PATH_EXPR, paymaster::ContractBuilder);
    blockchain.register_contract(ADDER_PATH_EXPR, adder::ContractBuilder);

    blockchain
}

struct PaymasterTestState {
    world: ScenarioWorld,
    callee_user_address: Address,
    paymaster_contract: PaymasterContract,
    relayer_address: Address,
    callee_sc_adder_contract: AdderContract,
}

impl PaymasterTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace().set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1))
                .put_account(
                    CALLER_ADDRESS_EXPR,
                    Account::new()
                        .nonce(1)
                        .balance(BALANCE)
                        .esdt_balance(PAYMASTER_TOKEN_ID_EXPR, BALANCE)
                        .esdt_balance(FEE_TOKEN_ID_EXPR, BALANCE)
                        .esdt_balance(ADDITIONAL_TOKEN_ID_EXPR, BALANCE),
                )
                .put_account(
                    CALLEE_USER_ADDRESS_EXPR,
                    Account::new().nonce(1).balance(BALANCE),
                )
                .put_account(RELAYER_ADDRESS_EXPR, Account::new().nonce(1).balance(0u32)),
        );

        let callee_user_address = AddressValue::from(CALLEE_USER_ADDRESS_EXPR).to_address();

        let relayer_address = AddressValue::from(RELAYER_ADDRESS_EXPR).to_address();
        let paymaster_contract = PaymasterContract::new(PAYMASTER_ADDRESS_EXPR);
        let callee_sc_adder_contract = AdderContract::new(CALLEE_SC_ADDER_ADDRESS_EXPR);
        // let callee_sc_adder_address = AddressValue::from(CALLEE_SC_ADDER_ADDRESS_EXPR).to_address();

        Self {
            world,
            callee_user_address,
            paymaster_contract,
            relayer_address,
            callee_sc_adder_contract,
        }
    }

    fn deploy_paymaster_contract(&mut self) -> &mut Self {
        let paymaster_code = self.world.code_expression(PAYMASTER_PATH_EXPR);

        self.world
            .set_state_step(SetStateStep::new().new_address(
                OWNER_ADDRESS_EXPR,
                1,
                PAYMASTER_ADDRESS_EXPR,
            ))
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER_ADDRESS_EXPR)
                    .code(paymaster_code)
                    .call(self.paymaster_contract.init()),
            );

        self
    }

    fn deploy_adder_contract(&mut self) -> &mut Self {
        let adder_code = self.world.code_expression(ADDER_PATH_EXPR);

        self.world
            .set_state_step(SetStateStep::new().new_address(
                OWNER_ADDRESS_EXPR,
                2,
                CALLEE_SC_ADDER_ADDRESS_EXPR,
            ))
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER_ADDRESS_EXPR)
                    .code(adder_code)
                    .call(self.callee_sc_adder_contract.init(INITIAL_ADD_VALUE)),
            );

        self
    }

    fn check_esdt_balance(
        &mut self,
        address_expr: &str,
        token_id_expr: &str,
        balance_expr: &str,
    ) -> &mut Self {
        self.world
            .check_state_step(CheckStateStep::new().put_account(
                address_expr,
                CheckAccount::new().esdt_balance(token_id_expr, balance_expr),
            ));

        self
    }
}

#[test]
fn test_deploy_paymasters() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();
}

#[test]
fn test_forward_call_no_fee_payment() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();

    state.world.sc_call(
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .call(state.paymaster_contract.forward_execution(
                state.relayer_address.clone(),
                state.callee_user_address.clone(),
                managed_buffer!(b"add"),
                MultiValueVec::<Vec<u8>>::new(),
            ))
            .expect(TxExpect::user_error("str:There is no fee for payment!")),
    );
}

#[test]
fn test_forward_call_user() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();

    state
        .world
        .sc_call(
            ScCallStep::new()
                .from(CALLER_ADDRESS_EXPR)
                .call(state.paymaster_contract.forward_execution(
                    state.relayer_address.clone(),
                    state.callee_user_address.clone(),
                    managed_buffer!(b"add"),
                    MultiValueVec::<Vec<u8>>::new(),
                ))
                .esdt_transfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
                .expect(TxExpect::ok()),
        )
        .check_state_step(CheckStateStep::new().put_account(
            RELAYER_ADDRESS_EXPR,
            CheckAccount::new().esdt_balance(FEE_TOKEN_ID_EXPR, FEE_AMOUNT),
        ));
}

#[test]
fn test_forward_call_sc_adder() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, PAYMASTER_TOKEN_ID_EXPR, BALANCE);

    state.world.sc_call(
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .esdt_transfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
            .esdt_transfer(PAYMASTER_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
            .call(state.paymaster_contract.forward_execution(
                state.relayer_address.clone(),
                state.callee_sc_adder_contract.to_address(),
                managed_buffer!(b"add"),
                MultiValueVec::from([top_encode_to_vec_u8_or_panic(&ADDITIONAL_ADD_VALUE)]),
            ))
            .expect(TxExpect::ok()),
    );

    let expected_adder_sum = INITIAL_ADD_VALUE + ADDITIONAL_ADD_VALUE;
    state.world.sc_query(
        ScQueryStep::new()
            .call(state.callee_sc_adder_contract.sum())
            .expect_value(SingleValue::from(BigUint::from(expected_adder_sum))),
    );
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
    state.check_esdt_balance(
        CALLEE_SC_ADDER_ADDRESS_EXPR,
        PAYMASTER_TOKEN_ID_EXPR,
        FEE_AMOUNT,
    );
}


#[test]
fn test_forward_call_sc_adder_multiple_payments() {
    let mut state = PaymasterTestState::new();
    state.deploy_paymaster_contract();
    state.deploy_adder_contract();

    state.check_esdt_balance(CALLER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, BALANCE);
    state.check_esdt_balance(CALLER_ADDRESS_EXPR, PAYMASTER_TOKEN_ID_EXPR, BALANCE);

    state.world.sc_call(
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .esdt_transfer(FEE_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
            .esdt_transfer(PAYMASTER_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
            .esdt_transfer(ADDITIONAL_TOKEN_ID_EXPR, 0, FEE_AMOUNT)
            .call(state.paymaster_contract.forward_execution(
                state.relayer_address.clone(),
                state.callee_sc_adder_contract.to_address(),
                managed_buffer!(b"add"),
                MultiValueVec::from([top_encode_to_vec_u8_or_panic(&ADDITIONAL_ADD_VALUE)]),
            ))
            .expect(TxExpect::ok()),
    );

    let expected_adder_sum = INITIAL_ADD_VALUE + ADDITIONAL_ADD_VALUE;
    state.world.sc_query(
        ScQueryStep::new()
            .call(state.callee_sc_adder_contract.sum())
            .expect_value(SingleValue::from(BigUint::from(expected_adder_sum))),
    );
    state.check_esdt_balance(RELAYER_ADDRESS_EXPR, FEE_TOKEN_ID_EXPR, FEE_AMOUNT);
    state.check_esdt_balance(
        CALLEE_SC_ADDER_ADDRESS_EXPR,
        PAYMASTER_TOKEN_ID_EXPR,
        FEE_AMOUNT,
    );
    state.check_esdt_balance(
        CALLEE_SC_ADDER_ADDRESS_EXPR,
        ADDITIONAL_TOKEN_ID_EXPR,
        FEE_AMOUNT,
    );
}
