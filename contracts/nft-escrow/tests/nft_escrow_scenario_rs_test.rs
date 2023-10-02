use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    todo!()
}

#[test]
#[ignore = "not supported"]
fn accept_rs() {
    multiversx_sc_scenario::run_rs("scenarios/accept.scen.json", world());
}

#[test]
#[ignore = "not supported"]
fn cancel_rs() {
    multiversx_sc_scenario::run_rs("scenarios/cancel.scen.json", world());
}

#[test]
#[ignore = "not supported"]
fn escrow_rs() {
    multiversx_sc_scenario::run_rs("scenarios/escrow.scen.json", world());
}

#[test]
#[ignore = "not supported"]
fn init_rs() {
    multiversx_sc_scenario::run_rs("scenarios/init.scen.json", world());
}

#[test]
#[ignore = "not supported"]
fn views_rs() {
    multiversx_sc_scenario::run_rs("scenarios/views.scen.json", world());
}
