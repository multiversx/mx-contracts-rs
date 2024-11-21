use core::iter::zip;

use multiversx_sc_scenario::{imports::MxscPath, DebugApi, ScenarioTxWhitebox, ScenarioWorld};
use rewards_distribution::{Bracket, RewardsDistribution, DIVISION_SAFETY_CONSTANT};

const OWNER: TestAddress = TestAddress::new("owner");
const REWARDS_DISTRIBUTION_ADDRESS: TestSCAddress = TestSCAddress::new("rewards_distribution");
const CODE_PATH: MxscPath = MxscPath::new("output/rewards-distribution.mxsc.json");

mod utils;

use multiversx_sc::imports::*;

fn world() -> ScenarioWorld {
    let mut blockchain: ScenarioWorld = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/rewards-distribution");
    blockchain.register_contract(CODE_PATH, rewards_distribution::ContractBuilder);
    blockchain
}

#[test]
fn test_compute_brackets() {
    let mut world = world();

    world.account(OWNER).nonce(1);
    world
        .account(REWARDS_DISTRIBUTION_ADDRESS)
        .nonce(1)
        .owner(OWNER)
        .code(CODE_PATH);

    world
        .tx()
        .from(OWNER)
        .to(REWARDS_DISTRIBUTION_ADDRESS)
        .whitebox(rewards_distribution::contract_obj, |sc| {
            let brackets: ManagedVec<DebugApi, Bracket> = utils::to_brackets(&[
                (10, 2_000),
                (90, 6_000),
                (400, 7_000),
                (2_500, 10_000),
                (25_000, 35_000),
                (72_000, 40_000),
            ])
            .into();

            let computed_brackets = sc.compute_brackets(brackets, 10_000);

            let expected_values = vec![
                (1, 2_000 * DIVISION_SAFETY_CONSTANT),
                (10, 6_000 * DIVISION_SAFETY_CONSTANT / (10 - 1)),
                (50, 7_000 * DIVISION_SAFETY_CONSTANT / (50 - 10)),
                (300, 10_000 * DIVISION_SAFETY_CONSTANT / (300 - 50)),
                (2_800, 35_000 * DIVISION_SAFETY_CONSTANT / (2_800 - 300)),
                (10_000, 40_000 * DIVISION_SAFETY_CONSTANT / (10_000 - 2_800)),
            ];

            assert_eq!(computed_brackets.len(), expected_values.len());
            for (computed, expected) in zip(computed_brackets.iter(), expected_values) {
                let (expected_end_index, expected_reward_percent) = expected;
                assert_eq!(computed.end_index, expected_end_index);
                assert_eq!(computed.nft_reward_percent, expected_reward_percent);
            }
        });
}
