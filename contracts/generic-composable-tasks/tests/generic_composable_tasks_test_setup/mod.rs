use generic_composable_tasks::GenericComposableTasks;
use multiversx_sc::types::{Address, EsdtLocalRole};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_token_id, rust_biguint, DebugApi,
};
use multiversx_wegld_swap_sc::EgldEsdtSwap;

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-12345";

pub const USER_EGLD_BALANCE: u64 = 1_000;

pub struct GenericCompTasksSetup<GenericCompTasksBuilder, EgldWrapperBuilder>
where
    GenericCompTasksBuilder:
        'static + Copy + Fn() -> generic_composable_tasks::ContractObj<DebugApi>,
    EgldWrapperBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub tasks_wrapper: ContractObjWrapper<
        generic_composable_tasks::ContractObj<DebugApi>,
        GenericCompTasksBuilder,
    >,
    pub wegld_swap_wrapper:
        ContractObjWrapper<multiversx_wegld_swap_sc::ContractObj<DebugApi>, EgldWrapperBuilder>,
}

impl<GenericCompTasksBuilder, EgldWrapperBuilder>
    GenericCompTasksSetup<GenericCompTasksBuilder, EgldWrapperBuilder>
where
    GenericCompTasksBuilder:
        'static + Copy + Fn() -> generic_composable_tasks::ContractObj<DebugApi>,
    EgldWrapperBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
{
    pub fn new(
        tasks_builder: GenericCompTasksBuilder,
        wegld_swap_builder: EgldWrapperBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let user_address = b_mock.create_user_account(&rust_biguint!(USER_EGLD_BALANCE));
        let owner_address = b_mock.create_user_account(&rust_biguint!(USER_EGLD_BALANCE));

        let tasks_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            tasks_builder,
            "generic composable tasks",
        );
        let wegld_swap_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            wegld_swap_builder,
            "wegld swap",
        );

        // init tasks sc
        b_mock
            .execute_tx(&owner_address, &tasks_wrapper, &rust_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        // init wegld swap sc
        b_mock
            .execute_tx(&owner_address, &wegld_swap_wrapper, &rust_zero, |sc| {
                sc.init(managed_token_id!(WEGLD_TOKEN_ID));
            })
            .assert_ok();

        // set wegld swap roles
        b_mock.set_esdt_local_roles(
            wegld_swap_wrapper.address_ref(),
            WEGLD_TOKEN_ID,
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        Self {
            b_mock,
            owner_address,
            user_address,
            tasks_wrapper,
            wegld_swap_wrapper,
        }
    }
}
