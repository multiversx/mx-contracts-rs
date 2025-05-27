#![allow(deprecated)]

use generic_composable_tasks::{
    call_dispatcher::{CallType, FunctionNameArgsPair, PaymentType, SingleCallArg},
    raw_call::common::PaymentsVec,
    GenericComposableTasks,
};
use multiversx_sc::{
    api::ManagedTypeApi,
    imports::TopDecode,
    types::{Address, EsdtLocalRole, EsdtTokenPayment, ManagedVec},
};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper, TxTokenTransfer},
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint, DebugApi,
};
use multiversx_wegld_swap_sc::EgldEsdtSwap;

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-12345";
pub static RAND_ESDT_TOKEN_ID: &[u8] = b"RANDESDT-12345";

pub const USER_BALANCE: u64 = 1_000;
pub const DEFAULT_GAS_LIMIT: u64 = 10_000_000;

pub static WRAP_EGLD_ENDPOINT_NAME: &[u8] = b"wrapEgld";
pub static UNWRAP_EGLD_ENDPOINT_NAME: &[u8] = b"unwrapEgld";

pub struct GenericCompTasksSetup<GenericCompTasksBuilder, EgldWrapperBuilder>
where
    GenericCompTasksBuilder:
        'static + Copy + Fn() -> generic_composable_tasks::ContractObj<DebugApi>,
    EgldWrapperBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub other_user_address: Address,
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
        let user_address = b_mock.create_user_account(&rust_biguint!(USER_BALANCE));
        let other_user_address = b_mock.create_user_account(&rust_biguint!(USER_BALANCE));
        let owner_address = b_mock.create_user_account(&rust_biguint!(USER_BALANCE));

        b_mock.set_esdt_balance(
            &user_address,
            RAND_ESDT_TOKEN_ID,
            &rust_biguint!(USER_BALANCE),
        );

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
            other_user_address,
            tasks_wrapper,
            wegld_swap_wrapper,
        }
    }
}

pub fn convert_transfers_to_managed<M: ManagedTypeApi>(
    esdt_transfers: Vec<TxTokenTransfer>,
) -> PaymentsVec<M> {
    let mut payments = PaymentsVec::new();
    for esdt_transfer in esdt_transfers {
        payments.push(EsdtTokenPayment::new(
            managed_token_id!(esdt_transfer.token_identifier),
            esdt_transfer.nonce,
            managed_biguint!(
                u64::top_decode(esdt_transfer.value.to_bytes_be().as_slice()).unwrap()
            ),
        ));
    }

    payments
}

pub fn build_egld_simple_transfer_data<M: ManagedTypeApi>(
    dest_address: &Address,
    egld_amount: u64,
) -> SingleCallArg<M> {
    (
        managed_address!(dest_address),
        PaymentType::Egld {
            amount: managed_biguint!(egld_amount),
        },
        CallType::SimpleTransfer,
        0,
        None,
    )
        .into()
}

pub fn build_esdt_simple_transfer_data<M: ManagedTypeApi>(
    dest_address: &Address,
    esdt_transfers: Vec<TxTokenTransfer>,
) -> SingleCallArg<M> {
    let mut arg = build_egld_simple_transfer_data(dest_address, 0);
    let payments = convert_transfers_to_managed(esdt_transfers);

    arg.0 .1 = PaymentType::FixedPayments {
        esdt_payments: payments,
    };

    arg
}

pub fn build_sync_call_egld_transfer_data<M: ManagedTypeApi>(
    dest_address: &Address,
    egld_amount: u64,
    function_name: &[u8],
    args: Vec<Vec<u8>>,
) -> SingleCallArg<M> {
    let mut managed_args = ManagedVec::new();
    for arg in args {
        managed_args.push(managed_buffer!(&arg));
    }

    (
        managed_address!(dest_address),
        PaymentType::Egld {
            amount: managed_biguint!(egld_amount),
        },
        CallType::Sync,
        DEFAULT_GAS_LIMIT,
        Some(FunctionNameArgsPair {
            function_name: managed_buffer!(function_name),
            args: managed_args,
        }),
    )
        .into()
}

pub fn build_sync_call_esdt_transfer_data<M: ManagedTypeApi>(
    dest_address: &Address,
    esdt_transfers: Vec<TxTokenTransfer>,
    function_name: &[u8],
    args: Vec<Vec<u8>>,
) -> SingleCallArg<M> {
    let mut arg = build_sync_call_egld_transfer_data(dest_address, 0, function_name, args);
    let payments = convert_transfers_to_managed(esdt_transfers);

    arg.0 .1 = PaymentType::FixedPayments {
        esdt_payments: payments,
    };

    arg
}
