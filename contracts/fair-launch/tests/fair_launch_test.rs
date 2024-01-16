#![allow(deprecated)]

mod tests_common;

use fair_launch::{common::CommonModule, initial_launch::InitialLaunchModule};
use multiversx_sc_scenario::managed_biguint;
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

#[test]
fn calculate_fee_test() {
    let mut fr_setup = FairLaunchSetup::new(fair_launch::contract_obj);
    fr_setup
        .b_mock
        .execute_query(&fr_setup.fl_wrapper, |sc| {
            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1_000), 4_000);
            let expected_fee = managed_biguint!(400);
            assert_eq!(fee, expected_fee);

            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1), 4_000);
            let expected_fee = managed_biguint!(1);
            assert_eq!(fee, expected_fee);

            let fee = sc.calculate_fee_rounded_up(&managed_biguint!(1_001), 4_000);
            let expected_fee = managed_biguint!(401);
            assert_eq!(fee, expected_fee);
        })
        .assert_ok();
}
