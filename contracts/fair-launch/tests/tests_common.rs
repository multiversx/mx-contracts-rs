use fair_launch::{common::Percentage, fair_launch_proxy, token_info::TokenInfoModule};
use multiversx_sc::{
    imports::StorageTokenWrapper,
    types::{ManagedBuffer, MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier},
};
use multiversx_sc_scenario::{imports::MxscPath, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld};
use pair_mock::PairMock;

pub const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("MYTOKEN-123456");
pub const OTHER_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("OTHER-123456");
pub const LAUNCH_DURATION_BLOCKS: u64 = 100;
pub const ACCOUNT_BUY_LIMIT: u64 = 2_000;
pub const TX_BUY_LIMIT: u64 = 1_000;
pub const BUY_FEE_PERCENTAGE_START: Percentage = 9_000;
pub const BUY_FEE_PERCENTAGE_END: Percentage = 1_000;
pub const SELL_FEE_PERCENTAGE_START: Percentage = 10_000;
pub const SELL_FEE_PERCENTAGE_END: Percentage = 5_000;
pub const CODE_PATH_FAIR_LAUNCH: MxscPath = MxscPath::new("output/fair-launch.mxsc.json");
pub const CODE_PATH_PAIR_MOCK: MxscPath = MxscPath::new("../pair-mock/output/pair-mock.mxsc.json");
pub const CODE_PATH_CROWDFUNDING: MxscPath =
    MxscPath::new("../crowdfunding-esdt/output/crowdfunding-esdt.mxsc.json");
pub const OWNER: TestAddress = TestAddress::new("owner");
pub const FIRST_ADDRESS: TestAddress = TestAddress::new("first-address");
pub const SECOND_ADDRESS: TestAddress = TestAddress::new("second-address");
pub const PAIR_MOCK_ADDRESS: TestSCAddress = TestSCAddress::new("pair-mock");
pub const FAIR_LAUNCH_ADDRESS: TestSCAddress = TestSCAddress::new("fair-launch");
pub const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowdfunding-esdt");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/fair-launch");
    blockchain.register_contract(CODE_PATH_FAIR_LAUNCH, fair_launch::ContractBuilder);
    blockchain.register_contract(CODE_PATH_PAIR_MOCK, pair_mock::ContractBuilder);
    blockchain.register_contract(CODE_PATH_CROWDFUNDING, crowdfunding_esdt::ContractBuilder);

    blockchain
}

pub struct FairLaunchSetup {
    pub world: ScenarioWorld,
}

impl FairLaunchSetup {
    pub fn new(token: Option<TestTokenIdentifier>, balance: u64) -> Self {
        let mut world = world();

        world.account(OWNER).nonce(1);
        world.account(SECOND_ADDRESS).nonce(1);

        match token {
            Some(t) => world
                .account(PAIR_MOCK_ADDRESS)
                .code(CODE_PATH_PAIR_MOCK)
                .owner(OWNER)
                .esdt_balance(t, balance),
            None => world
                .account(PAIR_MOCK_ADDRESS)
                .code(CODE_PATH_PAIR_MOCK)
                .owner(OWNER),
        };

        world
            .tx()
            .from(OWNER)
            .to(PAIR_MOCK_ADDRESS)
            .whitebox(pair_mock::contract_obj, |sc| {
                sc.init(
                    TOKEN_ID.to_token_identifier(),
                    OTHER_TOKEN_ID.to_token_identifier(),
                );
            });

        world
            .tx()
            .from(OWNER)
            .typed(fair_launch_proxy::FairLaunchProxy)
            .init(
                LAUNCH_DURATION_BLOCKS,
                ACCOUNT_BUY_LIMIT,
                TX_BUY_LIMIT,
                BUY_FEE_PERCENTAGE_START,
                BUY_FEE_PERCENTAGE_END,
                SELL_FEE_PERCENTAGE_START,
                SELL_FEE_PERCENTAGE_END,
            )
            .new_address(FAIR_LAUNCH_ADDRESS)
            .code(CODE_PATH_FAIR_LAUNCH)
            .run();

        world
            .tx()
            .from(OWNER)
            .to(FAIR_LAUNCH_ADDRESS)
            .whitebox(fair_launch::contract_obj, |sc| {
                sc.non_fungible_token()
                    .set_token_id(TOKEN_ID.to_token_identifier());
            });

        let mut pairs = MultiValueEncoded::new();
        pairs.push(
            (
                ManagedBuffer::from("swapTokensFixedInput"),
                4_000u32,
                false,
                0u32,
                false,
            )
                .into(),
        );

        world
            .tx()
            .from(OWNER)
            .to(FAIR_LAUNCH_ADDRESS)
            .typed(fair_launch_proxy::FairLaunchProxy)
            .add_exchange_endpoint(PAIR_MOCK_ADDRESS, pairs)
            .run();

        world
            .tx()
            .from(OWNER)
            .to(FAIR_LAUNCH_ADDRESS)
            .typed(fair_launch_proxy::FairLaunchProxy)
            .unpause_endpoint()
            .run();

        Self { world }
    }
}
