use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/nft-escrow");
    blockchain.register_contract(
        "mxsc:output/nft-escrow.mxsc.json",
        nft_escrow::ContractBuilder,
    );
    blockchain
}

#[test]
#[ignore = "not supported"]
fn accept_rs() {
    world().run("scenarios/accept.scen.json");
}

#[test]
#[ignore = "not supported"]
fn cancel_rs() {
    world().run("scenarios/cancel.scen.json");
}

#[test]
#[ignore = "not supported"]
fn escrow_rs() {
    world().run("scenarios/escrow.scen.json");
}

#[test]
#[ignore = "not supported"]
fn init_rs() {
    world().run("scenarios/init.scen.json");
}

#[test]
#[ignore = "not supported"]
fn views_rs() {
    world().run("scenarios/views.scen.json");
}
