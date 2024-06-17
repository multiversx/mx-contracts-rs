use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/potlock.mxsc.json", potlock::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/potlock.scen.json");
}
