#[test]
fn accept_go() {
    multiversx_sc_scenario::run_go("scenarios/accept.scen.json");
}

#[test]
fn cancel_go() {
    multiversx_sc_scenario::run_go("scenarios/cancel.scen.json");
}

#[test]
fn escrow_go() {
    multiversx_sc_scenario::run_go("scenarios/escrow.scen.json");
}

#[test]
fn init_go() {
    multiversx_sc_scenario::run_go("scenarios/init.scen.json");
}

#[test]
fn views_go() {
    multiversx_sc_scenario::run_go("scenarios/views.scen.json");
}
