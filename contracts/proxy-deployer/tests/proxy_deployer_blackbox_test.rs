use imports::{MxscPath, ReturnsResult, TestAddress, TestSCAddress};
use multiversx_sc::{
    codec::{multi_types::OptionalValue, top_encode_to_vec_u8_or_panic},
    types::{Address, CodeMetadata, ManagedAddress, ManagedBuffer, MultiValueEncoded},
};

use multiversx_sc_scenario::{api::StaticApi, scenario_model::*, *};

use adder::adder_proxy;
use proxy_deployer::proxy_deployer_proxy;

const PROXY_DEPLOYER_ADDRESS_EXPR: TestSCAddress = TestSCAddress::new("proxy_deployer");
const TEMPLATE_CONTRACT_ADDRESS_EXPR: TestSCAddress = TestSCAddress::new("template_contract");
const DEPLOYED_CONTRACT_ADDRESS_EXPR1: TestSCAddress = TestSCAddress::new("deployed_contract1");
const DEPLOYED_CONTRACT_ADDRESS_EXPR2: TestSCAddress = TestSCAddress::new("deployed_contract2");
const OWNER_ADDRESS_EXPR: TestAddress = TestAddress::new("owner");
const USER_ADDRESS_EXPR: TestAddress = TestAddress::new("user");

const PROXY_DEPLOYER_PATH_EXPR: MxscPath = MxscPath::new("output/proxy-deployer.mxsc.json");
const DEPLOYED_CONTRACT_PATH_EXPR: MxscPath = MxscPath::new("../adder/output/adder.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/proxy-deployer");
    blockchain.register_contract(PROXY_DEPLOYER_PATH_EXPR, proxy_deployer::ContractBuilder);
    blockchain.register_contract(DEPLOYED_CONTRACT_PATH_EXPR, adder::ContractBuilder);

    blockchain
}

struct ProxyDeployerTestState {
    world: ScenarioWorld,
    deployed_contracts: Vec<Address>,
}

impl ProxyDeployerTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace();
        world.account(OWNER_ADDRESS_EXPR).nonce(1);
        world.account(USER_ADDRESS_EXPR).nonce(1);

        Self {
            world,
            deployed_contracts: Vec::new(),
        }
    }

    fn deploy_proxy_deployer_contract(&mut self) -> &mut Self {
        self.world
            .new_address(OWNER_ADDRESS_EXPR, 1, PROXY_DEPLOYER_ADDRESS_EXPR);

        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .init(0u64)
            .code(PROXY_DEPLOYER_PATH_EXPR)
            .new_address(PROXY_DEPLOYER_ADDRESS_EXPR)
            .run();
        self.world
            .new_address(OWNER_ADDRESS_EXPR, 2, TEMPLATE_CONTRACT_ADDRESS_EXPR);
        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .typed(adder_proxy::AdderProxy)
            .init(0u64)
            .code(DEPLOYED_CONTRACT_PATH_EXPR)
            .new_address(TEMPLATE_CONTRACT_ADDRESS_EXPR)
            .run();
        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .to(PROXY_DEPLOYER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .add_template_address(TEMPLATE_CONTRACT_ADDRESS_EXPR.to_address())
            .run();

        self
    }

    fn deploy_contract(
        &mut self,
        user: TestAddress,
        creator_nonce: u64,
        deployed_address: TestSCAddress,
        template_address: TestSCAddress,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .new_address(PROXY_DEPLOYER_ADDRESS_EXPR, creator_nonce, deployed_address);
        let deploy_address = self
            .world
            .tx()
            .from(user)
            .to(PROXY_DEPLOYER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .contract_deploy(template_address.to_managed_address(), args)
            .returns(ReturnsResult)
            .run();
        self.deployed_contracts.push(deploy_address.to_address());

        self
    }

    fn upgrade_contract(
        &mut self,
        user: TestAddress,
        contract_address: &Address,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(user)
            .to(PROXY_DEPLOYER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .contract_upgrade(contract_address, args)
            .run();

        self
    }

    fn upgrade_by_template(
        &mut self,
        template_address: TestSCAddress,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        let gas = 0u64; // Gas is not taken into account

        self.world
            .tx()
            .from(OWNER_ADDRESS_EXPR)
            .to(PROXY_DEPLOYER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .upgrade_contracts_by_template(
                gas,
                OptionalValue::Some(template_address.to_managed_address()),
                args,
            )
            .run();

        self
    }

    fn call_endpoint(
        &mut self,
        user: TestAddress,
        contract_address: &Address,
        function_name: &str,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        let function = ManagedBuffer::from(function_name);
        self.world
            .tx()
            .from(user)
            .to(PROXY_DEPLOYER_ADDRESS_EXPR)
            .typed(proxy_deployer_proxy::ProxyDeployerProxy)
            .contract_call_by_address(
                ManagedAddress::from_address(contract_address),
                function,
                args,
            )
            .run();

        self
    }

    fn check_contract_storage(&mut self, deployed_address: TestSCAddress, expected_value: u64) {
        self.world
            .query()
            .to(deployed_address)
            .typed(adder_proxy::AdderProxy)
            .sum()
            .with_result(ExpectValue(expected_value))
            .run();
    }

    fn check_contract_metadata(
        &mut self,
        deployed_address: TestSCAddress,
        expected_value: CodeMetadata,
    ) {
        let metadata = BytesValue::from(expected_value.to_byte_array().as_ref());
        self.world
            .check_account(deployed_address)
            .code_metadata(metadata);
    }
}

#[test]
fn proxy_deployer_blackbox_test() {
    let mut state = ProxyDeployerTestState::new();
    state.deploy_proxy_deployer_contract();

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        0,
        DEPLOYED_CONTRACT_ADDRESS_EXPR1,
        TEMPLATE_CONTRACT_ADDRESS_EXPR,
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
    state.upgrade_contract(USER_ADDRESS_EXPR, &contract_address, upgrade_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 5u64);
}

#[test]
fn proxy_deployer_owner_bulk_upgrade() {
    let mut state = ProxyDeployerTestState::new();
    state.deploy_proxy_deployer_contract();

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        0,
        DEPLOYED_CONTRACT_ADDRESS_EXPR1,
        TEMPLATE_CONTRACT_ADDRESS_EXPR,
        deploy_args.clone(),
    );
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        1,
        DEPLOYED_CONTRACT_ADDRESS_EXPR2,
        TEMPLATE_CONTRACT_ADDRESS_EXPR,
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
    state.upgrade_by_template(TEMPLATE_CONTRACT_ADDRESS_EXPR, upgrade_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 5u64);
    // state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR2, 5u64);
}

#[test]
fn proxy_deployer_check_metadata_test() {
    let mut state = ProxyDeployerTestState::new();
    state.deploy_proxy_deployer_contract();

    state.check_contract_metadata(PROXY_DEPLOYER_ADDRESS_EXPR, CodeMetadata::UPGRADEABLE);

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(
        USER_ADDRESS_EXPR,
        0,
        DEPLOYED_CONTRACT_ADDRESS_EXPR1,
        TEMPLATE_CONTRACT_ADDRESS_EXPR,
        deploy_args,
    );
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 1u64);
    let contract_address = state.deployed_contracts[0].to_owned();

    state.check_contract_metadata(DEPLOYED_CONTRACT_ADDRESS_EXPR1, CodeMetadata::UPGRADEABLE);

    // Test endpoint call
    let mut call_args = MultiValueEncoded::new();
    call_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&9u64)));
    state.call_endpoint(USER_ADDRESS_EXPR, &contract_address, "add", call_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 10u64);

    // Test contract upgrade
    let mut upgrade_args = MultiValueEncoded::new();
    upgrade_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&5u64)));
    state.upgrade_contract(USER_ADDRESS_EXPR, &contract_address, upgrade_args);
    state.check_contract_storage(DEPLOYED_CONTRACT_ADDRESS_EXPR1, 5u64);
}
