#![allow(deprecated)]

mod tests_common;

use fair_launch::initial_launch::InitialLaunchModule;
use tests_common::*;

#[test]
fn init_test() {
    let _ = FairLaunchSetup::new(fair_launch::contract_obj);
}

#[test]
fn percentage_test() {
    let mut fr_setup = FairLaunchSetup::new(fair_launch::contract_obj);
    fr_setup.b_mock.set_block_nonce(10);
    fr_setup
        .b_mock
        .execute_query(&fr_setup.fl_wrapper, |sc| {
            let percentage =
                sc.get_fee_percentage(BUY_FEE_PERCENTAGE_START, BUY_FEE_PERCENTAGE_END);
            let expected_percentage = BUY_FEE_PERCENTAGE_START - 808; // (BUY_FEE_PERCENTAGE_END - BUY_FEE_PERCENTAGE_START) * 10 blocks / (100 blocks - 1) ~= 808
            assert_eq!(percentage, expected_percentage);
        })
        .assert_ok();
}
