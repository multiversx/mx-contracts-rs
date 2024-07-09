use multiversx_sc::contract_base::{CallableContract, ContractBase};
use multiversx_sc_scenario::DebugApi;

static CAN_EXECUTE_FN_NAME: &str = "canExecute";

#[derive(Clone)]
pub struct CanExecuteMock {}

impl ContractBase for CanExecuteMock {
    type Api = DebugApi;
}

impl CallableContract for CanExecuteMock {
    fn call(&self, fn_name: &str) -> bool {
        if fn_name == CAN_EXECUTE_FN_NAME {
            multiversx_sc::io::finish_multi::<DebugApi, _>(&true);

            return true;
        }

        false
    }
}

impl CanExecuteMock {
    pub fn new() -> Self {
        CanExecuteMock {}
    }
}
