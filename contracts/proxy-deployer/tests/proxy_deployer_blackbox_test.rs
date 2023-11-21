use multiversx_sc::{
    codec::{multi_types::OptionalValue, top_encode_to_vec_u8_or_panic},
    storage::mappers::SingleValue,
    types::{Address, ManagedAddress, ManagedBuffer, MultiValueEncoded},
};

use multiversx_sc_scenario::{api::StaticApi, num_bigint::BigUint, scenario_model::*, *};

use adder::ProxyTrait as _;
use proxy_deployer::{contract_interactions::ProxyTrait as _, ProxyTrait as _};

const PROXY_DEPLOYER_ADDRESS_EXPR: &str = "sc:proxy_deployer";
const TEMPLATE_CONTRACT_ADDRESS_EXPR: &str = "sc:template_contract";
const DEPLOYED_CONTRACT_ADDRESS_EXPR1: &str = "sc:deployed_contract1";
const DEPLOYED_CONTRACT_ADDRESS_EXPR2: &str = "sc:deployed_contract2";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const USER_ADDRESS_EXPR: &str = "address:user";

const PROXY_DEPLOYER_PATH_EXPR: &str = "file:output/proxy-deployer.wasm";
const DEPLOYED_CONTRACT_PATH_EXPR: &str = "file:~/contracts/adder/output/adder.wasm";

type ProxyDeployerContract = ContractInfo<proxy_deployer::Proxy<StaticApi>>;
type TemplateContract = ContractInfo<adder::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/proxy-deployer");

    blockchain.register_contract(PROXY_DEPLOYER_PATH_EXPR, proxy_deployer::ContractBuilder);
    blockchain.register_contract(DEPLOYED_CONTRACT_PATH_EXPR, adder::ContractBuilder);

    blockchain
}

struct ProxyDeployerTestState {
    world: ScenarioWorld,
    proxy_deployer_contract: ProxyDeployerContract,
    template_contract: TemplateContract,
    template_contract_address: Address,
    deployed_contracts: Vec<Address>,
}

impl ProxyDeployerTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace().set_state_step(
            SetStateStep::new().put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1)),
        );
        world.start_trace().set_state_step(
            SetStateStep::new().put_account(USER_ADDRESS_EXPR, Account::new().nonce(1)),
        );
        let proxy_deployer_contract = ProxyDeployerContract::new(PROXY_DEPLOYER_ADDRESS_EXPR);
        let template_contract = TemplateContract::new(TEMPLATE_CONTRACT_ADDRESS_EXPR);
        let template_contract_address =
            AddressValue::from(TEMPLATE_CONTRACT_ADDRESS_EXPR).to_address();

        Self {
            world,
            proxy_deployer_contract,
            template_contract,
            template_contract_address,
            deployed_contracts: vec![],
        }
    }

    fn deploy_proxy_deployer_contract(&mut self) -> &mut Self {
        let proxy_deployer_code = self.world.code_expression(PROXY_DEPLOYER_PATH_EXPR);
        let template_contract_code = self.world.code_expression(DEPLOYED_CONTRACT_PATH_EXPR);

        self.world
            .set_state_step(SetStateStep::new().new_address(
                OWNER_ADDRESS_EXPR,
                1,
                PROXY_DEPLOYER_ADDRESS_EXPR,
            ))
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER_ADDRESS_EXPR)
                    .code(proxy_deployer_code)
                    .call(self.proxy_deployer_contract.init(0u64)),
            )
            .set_state_step(SetStateStep::new().new_address(
                OWNER_ADDRESS_EXPR,
                2,
                TEMPLATE_CONTRACT_ADDRESS_EXPR,
            ))
            .sc_deploy(
                ScDeployStep::new()
                    .from(OWNER_ADDRESS_EXPR)
                    .code(template_contract_code)
                    .call(self.template_contract.init(BigUint::from(0u64))),
            );

        self
    }

    fn deploy_contract(
        &mut self,
        user: &str,
        creator_nonce: u64,
        deployed_address: &str,
        template_address: &Address,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .set_state_step(SetStateStep::new().new_address(
                PROXY_DEPLOYER_ADDRESS_EXPR,
                creator_nonce,
                deployed_address,
            ))
            .sc_call_use_result(
                ScCallStep::new()
                    .from(user)
                    .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                    .call(
                        self.proxy_deployer_contract
                            .contract_deploy(ManagedAddress::from_address(template_address), args),
                    ),
                |r: TypedResponse<ManagedAddress<StaticApi>>| {
                    self.deployed_contracts.push(r.result.unwrap().to_address());
                },
            );

        self
    }

    fn upgrade_contract(
        &mut self,
        user: &str,
        contract_address: &Address,
        template_address: &Address,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new()
                .from(user)
                .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                .call(self.proxy_deployer_contract.contract_upgrade(
                    ManagedAddress::from_address(contract_address),
                    ManagedAddress::from_address(template_address),
                    args,
                )),
        );

        self
    }

    fn upgrade_by_template(
        &mut self,
        template_address: &Address,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        let gas = 0u64; // Gas is not taken into account
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                .call(self.proxy_deployer_contract.upgrade_contracts_by_template(
                    gas,
                    OptionalValue::Some(ManagedAddress::from_address(template_address)),
                    OptionalValue::Some(args),
                )),
        );

        self
    }

    fn call_endpoint(
        &mut self,
        user: &str,
        contract_address: &Address,
        function_name: &str,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        let function = ManagedBuffer::from(function_name);
        self.world.sc_call(
            ScCallStep::new()
                .from(user)
                .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                .call(self.proxy_deployer_contract.contract_call_by_address(
                    ManagedAddress::from_address(contract_address),
                    function,
                    args,
                )),
        );

        self
    }

    fn check_contract_storage(&mut self, deployed_address: &str, expected_value: u64) {
        let mut deployed_contract = TemplateContract::new(deployed_address);

        self.world.sc_query(
            ScQueryStep::new()
                .call(deployed_contract.sum())
                .expect_value(SingleValue::from(BigUint::from(expected_value))),
        );
    }
}

#[test]
fn proxy_deployer_blackbox_test() {
    let mut state = ProxyDeployerTestState::new();
    state.deploy_proxy_deployer_contract();

    let template_address = state.template_contract_address.clone();

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        0,
        DEPLOYED_CONTRACT_ADDRESS_EXPR1,
        &template_address,
        deploy_args,
    );
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 1u64);
    let contract_address = state.deployed_contracts[0].to_owned();

    // Test endpoint call
    let mut call_args = MultiValueEncoded::new();
    call_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&9u64)));
    state.call_endpoint(USER_ADDRESS_EXPR, &contract_address, "add", call_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 10u64);

    // Test contract upgrade
    let mut upgrade_args = MultiValueEncoded::new();
    upgrade_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&5u64)));
    state.upgrade_contract(
        USER_ADDRESS_EXPR,
        &contract_address,
        &template_address,
        upgrade_args,
    );
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 5u64);
}

#[test]
fn proxy_deployer_owner_bulk_upgrade() {
    let mut state = ProxyDeployerTestState::new();
    state.deploy_proxy_deployer_contract();

    let template_address = state.template_contract_address.clone();

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        0,
        DEPLOYED_CONTRACT_ADDRESS_EXPR1,
        &template_address,
        deploy_args.clone(),
    );
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        1,
        DEPLOYED_CONTRACT_ADDRESS_EXPR2,
        &template_address,
        deploy_args,
    );
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 1u64);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR2, 1u64);
    let contract_address1 = state.deployed_contracts[0].to_owned();
    let contract_address2 = state.deployed_contracts[1].to_owned();

    // Test endpoint call
    let mut call_args = MultiValueEncoded::new();
    call_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&9u64)));
    state.call_endpoint(
        USER_ADDRESS_EXPR,
        &contract_address1,
        "add",
        call_args.clone(),
    );
    state.call_endpoint(USER_ADDRESS_EXPR, &contract_address2, "add", call_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 10u64);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR2, 10u64);

    // TODO - check complete output when upgrade from source contract is fully supported in blackbox testing
    // Test contract upgrade
    let mut upgrade_args = MultiValueEncoded::new();
    upgrade_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&5u64)));
    state.upgrade_by_template(&template_address, upgrade_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 5u64);
    // state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR2, 5u64);
}
