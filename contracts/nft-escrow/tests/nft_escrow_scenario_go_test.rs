use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn accept_go() {
    world().run("scenarios/accept.scen.json");
}

#[test]
fn cancel_go() {
    world().run("scenarios/cancel.scen.json");
}

#[test]
fn escrow_go() {
    world().run("scenarios/escrow.scen.json");
}

#[test]
fn init_go() {
    world().run("scenarios/init.scen.json");
}

#[test]
fn views_go() {
    world().run("scenarios/views.scen.json");
}
