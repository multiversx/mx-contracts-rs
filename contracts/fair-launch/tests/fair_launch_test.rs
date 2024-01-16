#![allow(deprecated)]

mod tests_common;

use tests_common::*;

#[test]
fn init_test() {
    let _ = FairLaunchSetup::new(fair_launch::contract_obj);
}
