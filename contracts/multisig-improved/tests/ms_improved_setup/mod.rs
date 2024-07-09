use adder::Adder;
use multisig_improved::Multisig;
use multiversx_sc::types::{Address, MultiValueEncoded};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_address, managed_biguint, rust_biguint, DebugApi,
};

pub mod can_execute_mock;

pub struct MsImprovedSetup<MsImprovedBuilder, AdderBuilder>
where
    MsImprovedBuilder: 'static + Copy + Fn() -> multisig_improved::ContractObj<DebugApi>,
    AdderBuilder: 'static + Copy + Fn() -> adder::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub first_board_member: Address,
    pub second_board_member: Address,
    pub ms_owner: Address,
    pub ms_wrapper: ContractObjWrapper<multisig_improved::ContractObj<DebugApi>, MsImprovedBuilder>,
    pub adder_wrapper: ContractObjWrapper<adder::ContractObj<DebugApi>, AdderBuilder>,
}

impl<MsImprovedBuilder, AdderBuilder> MsImprovedSetup<MsImprovedBuilder, AdderBuilder>
where
    MsImprovedBuilder: 'static + Copy + Fn() -> multisig_improved::ContractObj<DebugApi>,
    AdderBuilder: 'static + Copy + Fn() -> adder::ContractObj<DebugApi>,
{
    pub fn new(ms_builder: MsImprovedBuilder, adder_builder: AdderBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let first_board_member = b_mock.create_user_account(&rust_zero);
        let second_board_member = b_mock.create_user_account(&rust_zero);
        let ms_owner = b_mock.create_user_account(&rust_zero);
        let adder_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&first_board_member),
            adder_builder,
            "adder",
        );
        let ms_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&ms_owner), ms_builder, "multisig");

        // init adder
        b_mock
            .execute_tx(&first_board_member, &adder_wrapper, &rust_zero, |sc| {
                sc.init(managed_biguint!(0));
            })
            .assert_ok();

        // init multisig
        b_mock
            .execute_tx(&ms_owner, &ms_wrapper, &rust_zero, |sc| {
                let mut board = MultiValueEncoded::new();
                board.push(managed_address!(&first_board_member));
                board.push(managed_address!(&second_board_member));

                sc.init(2, board);
            })
            .assert_ok();

        Self {
            b_mock,
            first_board_member,
            second_board_member,
            ms_owner,
            ms_wrapper,
            adder_wrapper,
        }
    }
}
