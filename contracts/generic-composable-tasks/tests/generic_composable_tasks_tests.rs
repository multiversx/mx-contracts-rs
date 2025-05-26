use generic_composable_tasks_test_setup::GenericCompTasksSetup;

pub mod generic_composable_tasks_test_setup;

#[test]
fn setup_test() {
    let _ = GenericCompTasksSetup::new(
        generic_composable_tasks::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
    );
}
