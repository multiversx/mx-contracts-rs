use multiversx_sc::{
    codec::top_encode_to_vec_u8_or_panic,
    storage::mappers::SingleValue,
    types::{Address, ManagedAddress, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{api::StaticApi, num_bigint::BigUint, scenario_model::*, *};

use adder::ProxyTrait as _;
use proxy_deployer::{
    config::ProxyTrait as _, contract_interactions::ProxyTrait as _, ProxyTrait as _,
};

const PROXY_DEPLOYER_ADDRESS_EXPR: &str = "sc:proxy_deployer";
const TEMPLATE_CONTRACT_ADDRESS_EXPR: &str = "sc:template_contract";
const DEPLOYED_CONTRACT_ADDRESS_EXPR: &str = "sc:deployed_contract";
const OWNER_ADDRESS_EXPR: &str = "address:owner";

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
    template_contracts_ids: Vec<u64>,
    deployed_contracts: Vec<Address>,
}

impl ProxyDeployerTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace().set_state_step(
            SetStateStep::new().put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1)),
        );
        let proxy_deployer_contract = ProxyDeployerContract::new(PROXY_DEPLOYER_ADDRESS_EXPR);
        let template_contract = TemplateContract::new(TEMPLATE_CONTRACT_ADDRESS_EXPR);

        Self {
            world,
            proxy_deployer_contract,
            template_contract,
            template_contracts_ids: vec![],
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
                    .call(self.proxy_deployer_contract.init()),
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
            )
            .sc_call_use_result(
                ScCallStep::new().from(OWNER_ADDRESS_EXPR).call(
                    self.proxy_deployer_contract.add_contract_template(
                        ManagedAddress::from_address(&self.template_contract.to_address()),
                    ),
                ),
                |r: TypedResponse<u64>| {
                    self.template_contracts_ids.push(r.result.unwrap());
                },
            );

        self
    }

    fn deploy_contract(
        &mut self,
        template_address_id: u64,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .set_state_step(SetStateStep::new().new_address(
                PROXY_DEPLOYER_ADDRESS_EXPR,
                0,
                DEPLOYED_CONTRACT_ADDRESS_EXPR,
            ))
            .sc_call_use_result(
                ScCallStep::new()
                    .from(OWNER_ADDRESS_EXPR)
                    .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                    .call(
                        self.proxy_deployer_contract
                            .contract_deploy(template_address_id, args),
                    ),
                |r: TypedResponse<ManagedAddress<StaticApi>>| {
                    self.deployed_contracts.push(r.result.unwrap().to_address());
                },
            );

        self
    }

    fn upgrade_contract(
        &mut self,
        contract_address: &Address,
        template_address_id: u64,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                .call(self.proxy_deployer_contract.contract_upgrade(
                    ManagedAddress::from_address(contract_address),
                    template_address_id,
                    args,
                )),
        );

        self
    }

    fn call_endpoint(
        &mut self,
        contract_address: &Address,
        function_name: &str,
        args: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        let function = ManagedBuffer::from(function_name);
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .to(PROXY_DEPLOYER_ADDRESS_EXPR)
                .call(self.proxy_deployer_contract.call_contract_endpoint(
                    ManagedAddress::from_address(contract_address),
                    function,
                    args,
                )),
        );

        self
    }

    fn check_contract_storage(&mut self, expected_value: u64) {
        let mut deployed_contract = TemplateContract::new(DEPLOYED_CONTRACT_ADDRESS_EXPR);

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

    // Test contract deploy
    let mut deploy_args = MultiValueEncoded::new();
    deploy_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&1u64)));
    state.deploy_contract(state.template_contracts_ids[0], deploy_args);
    state.check_contract_storage(1u64);
    let contract_address = state.deployed_contracts[0].to_owned();

    // Test endpoint call
    let mut call_args = MultiValueEncoded::new();
    call_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&9u64)));
    state.call_endpoint(&contract_address, "add", call_args);
    state.check_contract_storage(10u64);

    // Test contract upgrade
    state.world.dump_state_step();
    let mut upgrade_args = MultiValueEncoded::new();
    upgrade_args.push(ManagedBuffer::from(top_encode_to_vec_u8_or_panic(&5u64)));
    state.upgrade_contract(
        &contract_address,
        state.template_contracts_ids[0],
        upgrade_args,
    );
    state.check_contract_storage(5u64);
}
