use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    todo!()
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
