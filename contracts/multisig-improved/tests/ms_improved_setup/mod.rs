use adder::Adder;
use multisig_improved::{
    common_types::{
        action::{ActionId, Nonce},
        signature::{ActionType, SignatureArg, SignatureType},
        user_role::UserRole,
    },
    external::views::ViewsModule,
    ms_endpoints::{
        perform::PerformEndpointsModule, propose::ProposeEndpointsModule, sign::SignEndpointsModule,
    },
    Multisig,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{Address, CodeMetadata, FunctionCall, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_address, managed_biguint, managed_buffer, rust_biguint, DebugApi,
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

    pub fn propose_add_board_member(&mut self, board_member: &Address) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    action_id = sc.propose_add_board_member(
                        managed_address!(board_member),
                        OptionalValue::None,
                    );
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_add_proposer(&mut self, proposer: &Address) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    action_id =
                        sc.propose_add_proposer(managed_address!(proposer), OptionalValue::None);
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_change_quorum(&mut self, new_quorum: usize) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    action_id = sc.propose_change_quorum(new_quorum, OptionalValue::None);
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_transfer_execute(
        &mut self,
        to: &Address,
        egld_amount: u64,
        function_name: &[u8],
        args: Vec<&[u8]>,
    ) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut function_call = FunctionCall::new(function_name);
                    for arg in args {
                        function_call = function_call.argument(&arg);
                    }

                    action_id = sc
                        .propose_transfer_execute(
                            managed_address!(to),
                            managed_biguint!(egld_amount),
                            None,
                            function_call,
                            OptionalValue::None,
                        )
                        .into_option()
                        .unwrap();
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_async_call(
        &mut self,
        to: &Address,
        egld_amount: u64,
        function_name: &[u8],
        args: Vec<&[u8]>,
    ) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut function_call = FunctionCall::new(function_name);
                    for arg in args {
                        function_call = function_call.argument(&arg);
                    }

                    action_id = sc.propose_async_call(
                        managed_address!(to),
                        managed_biguint!(egld_amount),
                        None,
                        function_call,
                        OptionalValue::None,
                    );
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_remove_user(&mut self, user: &Address) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    action_id = sc.propose_remove_user(managed_address!(user), OptionalValue::None);
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_sc_deploy_from_source(
        &mut self,
        egld_amount: u64,
        source: &Address,
        code_metadata: CodeMetadata,
        arguments: Vec<&[u8]>,
    ) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut args = MultiValueEncoded::new();
                    for arg in arguments {
                        args.push(managed_buffer!(arg));
                    }

                    action_id = sc.propose_sc_deploy_from_source(
                        managed_biguint!(egld_amount),
                        managed_address!(source),
                        code_metadata,
                        None,
                        args,
                    );
                },
            )
            .assert_ok();

        action_id
    }

    pub fn propose_sc_upgrade_from_source(
        &mut self,
        sc_address: &Address,
        egld_amount: u64,
        source: &Address,
        code_metadata: CodeMetadata,
        arguments: Vec<&[u8]>,
    ) -> ActionId {
        let mut action_id = 0;

        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut args = MultiValueEncoded::new();
                    for arg in arguments {
                        args.push(managed_buffer!(arg));
                    }

                    action_id = sc.propose_sc_upgrade_from_source(
                        managed_address!(sc_address),
                        managed_biguint!(egld_amount),
                        managed_address!(source),
                        code_metadata,
                        None,
                        args,
                    );
                },
            )
            .assert_ok();

        action_id
    }

    pub fn perform(&mut self, action_id: ActionId) {
        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let _ = sc.perform_action_endpoint(action_id);
                },
            )
            .assert_ok();
    }

    pub fn perform_and_expect_err(&mut self, action_id: ActionId, err_message: &str) {
        self.b_mock
            .execute_tx(
                &self.first_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let _ = sc.perform_action_endpoint(action_id);
                },
            )
            .assert_user_error(err_message);
    }

    pub fn sign(&mut self, action_id: ActionId, signer_nonce: Nonce) {
        let signer_addr = self.second_board_member.clone();

        self.b_mock
            .execute_tx(
                &self.second_board_member,
                &self.ms_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut signatures = MultiValueEncoded::new();
                    signatures.push(SignatureArg {
                        user_address: managed_address!(&signer_addr),
                        nonce: signer_nonce,
                        action_type: ActionType::SimpleAction,
                        raw_sig_bytes: managed_buffer!(b"signature"),
                        signature_type: SignatureType::Ed25519, // unused
                    });

                    sc.sign(action_id, signatures);
                },
            )
            .assert_ok();
    }

    pub fn expect_user_role(&mut self, user: &Address, expected_user_role: UserRole) {
        self.b_mock
            .execute_query(&self.ms_wrapper, |sc| {
                let actual_user_role = sc.user_role(managed_address!(user));
                assert_eq!(actual_user_role, expected_user_role);
            })
            .assert_ok();
    }
}
