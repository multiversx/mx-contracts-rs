pub mod ms_improved_setup;

use ms_improved_setup::*;

#[test]
fn init_test() {
    let _ = MsImprovedSetup::new(multisig_improved::contract_obj, adder::contract_obj);
}
