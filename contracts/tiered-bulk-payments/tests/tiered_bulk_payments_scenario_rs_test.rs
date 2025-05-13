use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // blockchain.set_current_dir_from_workspace("relative path to your workspace, if applicable");
    blockchain.register_contract("mxsc:output/tiered-bulk-payments.mxsc.json", tiered_bulk_payments::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/tiered_bulk_payments.scen.json");
}
