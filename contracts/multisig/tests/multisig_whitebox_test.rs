use adder::Adder;
use factorial::Factorial;
use multisig::{
    action::GasLimit, multisig_perform::MultisigPerformModule,
    multisig_propose::MultisigProposeModule, multisig_sign::MultisigSignModule,
    user_role::UserRole, Multisig,
};
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{
        Address, BigUint, BoxedBytes, CodeMetadata, FunctionCall, ManagedBuffer, ManagedVec,
        TestAddress, TestSCAddress,
    },
};
use multiversx_sc_scenario::{
    imports::MxscPath, managed_address, managed_biguint, rust_biguint, ExpectMessage,
    ReturnsHandledOrError, ScenarioTxWhitebox, ScenarioWorld,
};

const OWNER: TestAddress = TestAddress::new("owner");
const PROPOSER: TestAddress = TestAddress::new("proposer");
const BOARD_MEMBER: TestAddress = TestAddress::new("board-member");
const MULTISIG: TestSCAddress = TestSCAddress::new("multisig");
const CODE_PATH: MxscPath = MxscPath::new("output/multisig.mxsc.json");
const QUORUM_SIZE: usize = 1;

type RustBigUint = num_bigint::BigUint;

pub enum ActionRaw {
    _Nothing,
    AddBoardMember(Address),
    AddProposer(Address),
    RemoveUser(Address),
    ChangeQuorum(usize),
    SendTransferExecute(CallActionDataRaw),
    SendAsyncCall(CallActionDataRaw),
    SCDeployFromSource {
        amount: RustBigUint,
        source: Address,
        code_metadata: CodeMetadata,
        arguments: Vec<BoxedBytes>,
    },
    SCUpgradeFromSource {
        sc_address: Address,
        amount: RustBigUint,
        source: Address,
        code_metadata: CodeMetadata,
        arguments: Vec<BoxedBytes>,
    },
}

pub struct CallActionDataRaw {
    pub to: Address,
    pub egld_amount: RustBigUint,
    pub endpoint_name: BoxedBytes,
    pub arguments: Vec<BoxedBytes>,
}

fn world() -> ScenarioWorld {
    let mut blockchain: ScenarioWorld = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/multisig");
    blockchain.register_contract(CODE_PATH, multisig::ContractBuilder);
    blockchain
}

fn setup() -> ScenarioWorld {
    // setup
    let mut world = world();

    world
        .account(OWNER)
        .nonce(1)
        .new_address(OWNER, 1, MULTISIG);
    world.account(PROPOSER).nonce(1).balance(100_000_000u64);
    world.account(BOARD_MEMBER).nonce(1);

    // init multisig
    world
        .tx()
        .from(OWNER)
        .raw_deploy()
        .code(CODE_PATH)
        .new_address(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let mut board_members = ManagedVec::new();
            board_members.push(BOARD_MEMBER.to_managed_address());

            sc.init(QUORUM_SIZE, board_members.into());
            sc.change_user_role(0, PROPOSER.to_managed_address(), UserRole::Proposer);
        });

    world
}

#[test]
fn test_init() {
    setup();
}

fn call_propose(
    world: &mut ScenarioWorld,
    action: ActionRaw,
    expected_message: Option<&str>,
) -> usize {
    let egld_amount = match &action {
        ActionRaw::SendTransferExecute(call_data) => call_data.egld_amount.clone(),
        ActionRaw::SendAsyncCall(call_data) => call_data.egld_amount.clone(),
        ActionRaw::SCDeployFromSource { amount, .. } => amount.clone(),
        ActionRaw::SCUpgradeFromSource { amount, .. } => amount.clone(),
        _ => rust_biguint!(0),
    };

    let mut action_id = 0;

    let result = world
        .tx()
        .from(PROPOSER)
        .to(MULTISIG)
        .egld(BigUint::from(egld_amount))
        .returns(ReturnsHandledOrError::new())
        .whitebox(multisig::contract_obj, |sc| {
            action_id = match action {
                ActionRaw::_Nothing => panic!("Invalid action"),
                ActionRaw::AddBoardMember(addr) => {
                    sc.propose_add_board_member(managed_address!(&addr))
                }
                ActionRaw::AddProposer(addr) => sc.propose_add_proposer(managed_address!(&addr)),
                ActionRaw::RemoveUser(addr) => sc.propose_remove_user(managed_address!(&addr)),
                ActionRaw::ChangeQuorum(new_size) => sc.propose_change_quorum(new_size),
                ActionRaw::SendTransferExecute(call_data) => sc.propose_transfer_execute(
                    managed_address!(&call_data.to),
                    BigUint::from_bytes_be(&call_data.egld_amount.to_bytes_be()),
                    Option::<GasLimit>::None,
                    FunctionCall {
                        function_name: call_data.endpoint_name.into(),
                        arg_buffer: call_data.arguments.into(),
                    },
                ),
                ActionRaw::SendAsyncCall(call_data) => sc.propose_async_call(
                    managed_address!(&call_data.to),
                    BigUint::from_bytes_be(&call_data.egld_amount.to_bytes_be()),
                    Option::<GasLimit>::None,
                    FunctionCall {
                        function_name: call_data.endpoint_name.into(),
                        arg_buffer: call_data.arguments.into(),
                    },
                ),
                ActionRaw::SCDeployFromSource {
                    amount,
                    source,
                    code_metadata,
                    arguments,
                } => sc.propose_sc_deploy_from_source(
                    BigUint::from_bytes_be(&amount.to_bytes_be()),
                    managed_address!(&source),
                    code_metadata,
                    boxed_bytes_vec_to_managed(arguments).into(),
                ),
                ActionRaw::SCUpgradeFromSource {
                    sc_address,
                    amount,
                    source,
                    code_metadata,
                    arguments,
                } => sc.propose_sc_upgrade_from_source(
                    managed_address!(&sc_address),
                    BigUint::from_bytes_be(&amount.to_bytes_be()),
                    managed_address!(&source),
                    code_metadata,
                    boxed_bytes_vec_to_managed(arguments).into(),
                ),
            }
        });

    if let Err(err) = result {
        assert_eq!(expected_message.unwrap_or_default(), err.message);
    }

    action_id
}

#[test]
fn test_add_board_member() {
    let mut world = setup();

    let new_board_member: TestAddress = TestAddress::new("new-board-member");
    world.account(new_board_member).nonce(1);

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role before
            let user_role = sc.user_role(new_board_member.to_managed_address());
            assert_eq!(user_role, UserRole::None);
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::AddBoardMember(new_board_member.to_address()),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            sc.sign(action_id);
        });

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role after
            let user_role = sc.user_role(new_board_member.to_managed_address());
            assert_eq!(user_role, UserRole::BoardMember);

            let board_members = sc.get_all_board_members().to_vec();

            assert_eq!(*board_members.get(0), BOARD_MEMBER);
            assert_eq!(*board_members.get(1), new_board_member);
        });
}

#[test]
fn test_add_proposer() {
    let mut world = setup();

    let new_proposer: TestAddress = TestAddress::new("new-proposer");
    world.account(new_proposer).nonce(1);

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role before
            let user_role = sc.user_role(new_proposer.to_managed_address());

            assert_eq!(user_role, UserRole::None);
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::AddProposer(new_proposer.to_address()),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role after
            let user_role = sc.user_role(new_proposer.to_managed_address());

            assert_eq!(user_role, UserRole::Proposer);

            let proposers = sc.get_all_proposers().to_vec();
            assert_eq!(*proposers.get(0), PROPOSER);
            assert_eq!(*proposers.get(1), new_proposer);
        });
}

#[test]
fn test_remove_proposer() {
    let mut world = setup();

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role before
            let user_role = sc.user_role(PROPOSER.to_managed_address());

            assert_eq!(user_role, UserRole::Proposer);
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::RemoveUser(PROPOSER.to_address()),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    world
        .query()
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            // check role after
            let user_role = sc.user_role(PROPOSER.to_managed_address());

            assert_eq!(user_role, UserRole::None);

            let proposers = sc.get_all_proposers();
            assert!(proposers.is_empty());
        });
}

#[test]
fn test_try_remove_all_board_members() {
    let mut world = setup();

    let action_id = call_propose(
        &mut world,
        ActionRaw::RemoveUser(BOARD_MEMBER.to_address()),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .returns(ExpectMessage("quorum cannot exceed board size"))
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });
}

#[test]
fn test_change_quorum() {
    let mut world = setup();

    let new_quorum_size = 2;

    // try change quorum > board size
    let action_id = call_propose(&mut world, ActionRaw::ChangeQuorum(new_quorum_size), None);

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .returns(ExpectMessage("quorum cannot exceed board size"))
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    // try discard before unsigning
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .returns(ExpectMessage("cannot discard action with valid signatures"))
        .whitebox(multisig::contract_obj, |sc| sc.discard_action(action_id));

    // unsign and discard action
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.unsign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.discard_action(action_id));

    // try sign discarded action
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .returns(ExpectMessage("action does not exist"))
        .whitebox(multisig::contract_obj, |sc| {
            sc.sign(action_id);
        });

    // add another board member
    let new_board_member: TestAddress = TestAddress::new("new-board-member");
    world.account(new_board_member).nonce(1);

    let action_id = call_propose(
        &mut world,
        ActionRaw::AddBoardMember(new_board_member.to_address()),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    // change quorum to 2
    let action_id = call_propose(&mut world, ActionRaw::ChangeQuorum(new_quorum_size), None);

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });
}

#[test]
fn test_transfer_execute_to_user() {
    let mut world = setup();

    let new_user: TestAddress = TestAddress::new("new-user");
    world.account(new_user).nonce(1);

    let egld_amount: u64 = 100;

    world
        .tx()
        .from(PROPOSER)
        .to(MULTISIG)
        .egld(egld_amount)
        .whitebox(multisig::contract_obj, |sc| {
            sc.deposit();
        });
    world.check_account(MULTISIG).balance(egld_amount);

    // failed attempt
    call_propose(
        &mut world,
        ActionRaw::SendTransferExecute(CallActionDataRaw {
            to: new_user.to_address(),
            egld_amount: rust_biguint!(0),
            endpoint_name: BoxedBytes::empty(),
            arguments: Vec::new(),
        }),
        Some("proposed action has no effect"),
    );

    // propose
    let action_id = call_propose(
        &mut world,
        ActionRaw::SendTransferExecute(CallActionDataRaw {
            to: new_user.to_address(),
            egld_amount: rust_biguint!(egld_amount),
            endpoint_name: BoxedBytes::empty(),
            arguments: Vec::new(),
        }),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    world.check_account(new_user).balance(egld_amount);
}

#[test]
fn test_transfer_execute_sc_all() {
    let mut world = setup();

    let adder_owner: TestAddress = TestAddress::new("adder-owner");
    let adder_address: TestSCAddress = TestSCAddress::new("adder");
    let adder_code_path: MxscPath = MxscPath::new("test-contracts/adder.mxsc.json");

    world.register_contract(adder_code_path, adder::ContractBuilder);
    world
        .account(adder_owner)
        .nonce(1)
        .new_address(adder_owner, 1, adder_address);

    world
        .tx()
        .raw_deploy()
        .from(adder_owner)
        .code(adder_code_path)
        .new_address(adder_address)
        .whitebox(adder::contract_obj, |sc| {
            sc.init(BigUint::from(5u64));
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::SendTransferExecute(CallActionDataRaw {
            to: adder_address.to_address(),
            egld_amount: 0u64.into(),
            endpoint_name: BoxedBytes::from(&b"add"[..]),
            arguments: vec![BoxedBytes::from(&[5u8][..])],
        }),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });

    world
        .query()
        .to(adder_address)
        .whitebox(adder::contract_obj, |sc| {
            let actual_sum = sc.sum().get();
            let expected_sum = managed_biguint!(10);
            assert_eq!(actual_sum, expected_sum);
        });
}

#[test]
fn test_async_call_to_sc() {
    let mut world = setup();

    let adder_owner: TestAddress = TestAddress::new("adder-owner");
    let adder_address: TestSCAddress = TestSCAddress::new("adder");
    let adder_code_path: MxscPath = MxscPath::new("test-contracts/adder.mxsc.json");

    world.register_contract(adder_code_path, adder::ContractBuilder);
    world
        .account(adder_owner)
        .nonce(1)
        .new_address(adder_owner, 1, adder_address);

    world
        .tx()
        .raw_deploy()
        .from(adder_owner)
        .code(adder_code_path)
        .new_address(adder_address)
        .whitebox(adder::contract_obj, |sc| {
            sc.init(BigUint::from(5u64));
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::SendAsyncCall(CallActionDataRaw {
            to: adder_address.to_address(),
            egld_amount: 0u64.into(),
            endpoint_name: BoxedBytes::from(&b"add"[..]),
            arguments: vec![BoxedBytes::from(&[5u8][..])],
        }),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });
    world
        .query()
        .to(adder_address)
        .whitebox(adder::contract_obj, |sc| {
            let actual_sum = sc.sum().get();
            let expected_sum = managed_biguint!(10);
            assert_eq!(actual_sum, expected_sum);
        });
}

#[test]
fn test_deploy_and_upgrade_from_source() {
    let mut world = setup();

    let new_adder_address: TestSCAddress = TestSCAddress::new("new-adder");
    let adder_owner: TestAddress = TestAddress::new("adder-owner");
    let adder_address: TestSCAddress = TestSCAddress::new("adder");
    let adder_path: MxscPath = MxscPath::new("test-contracts/adder.mxsc.json");

    world.register_contract(adder_path, adder::ContractBuilder);
    world
        .account(adder_owner)
        .nonce(1)
        .new_address(adder_owner, 1, adder_address)
        .new_address(MULTISIG, 0, new_adder_address);

    world
        .tx()
        .raw_deploy()
        .from(adder_owner)
        .code(adder_path)
        .new_address(adder_address)
        .whitebox(adder::contract_obj, |sc| {
            sc.init(BigUint::from(5u64));
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::SCDeployFromSource {
            amount: 0u64.into(),
            source: adder_address.to_address(),
            code_metadata: CodeMetadata::all(),
            arguments: vec![BoxedBytes::from(&[5u8][..])],
        },
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let opt_address = sc.perform_action_endpoint(action_id);
            let addr = opt_address.into_option().unwrap().to_address();
            assert_eq!(new_adder_address.to_address(), addr);
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::SendTransferExecute(CallActionDataRaw {
            to: new_adder_address.to_address(),
            egld_amount: 0u64.into(),
            endpoint_name: BoxedBytes::from(&b"add"[..]),
            arguments: vec![BoxedBytes::from(&[5u8][..])],
        }),
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });
    world
        .query()
        .to(new_adder_address)
        .whitebox(adder::contract_obj, |sc| {
            let actual_sum = sc.sum().get();
            let expected_sum = managed_biguint!(10);
            assert_eq!(actual_sum, expected_sum);
        });

    let factorial_address: TestSCAddress = TestSCAddress::new("factorial");
    let factorial_code: MxscPath = MxscPath::new("test-contracts/factorial.mxsc.json");

    world.register_contract(factorial_code, factorial::ContractBuilder);
    world
        .tx()
        .raw_deploy()
        .from(OWNER)
        .code(factorial_code)
        .new_address(factorial_address)
        .whitebox(factorial::contract_obj, |sc| {
            sc.init();
        });

    let action_id = call_propose(
        &mut world,
        ActionRaw::SCUpgradeFromSource {
            source: factorial_address.to_address(),
            amount: 0u64.into(),
            code_metadata: CodeMetadata::all(),
            arguments: Vec::new(),
            sc_address: adder_address.to_address(),
        },
        None,
    );

    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| sc.sign(action_id));
    world
        .tx()
        .from(BOARD_MEMBER)
        .to(MULTISIG)
        .whitebox(multisig::contract_obj, |sc| {
            let _ = sc.perform_action_endpoint(action_id);
        });
    world.check_account(adder_address).code(factorial_code);
}

fn boxed_bytes_vec_to_managed<M: ManagedTypeApi>(
    raw_vec: Vec<BoxedBytes>,
) -> ManagedVec<M, ManagedBuffer<M>> {
    let mut managed = ManagedVec::new();
    for elem in raw_vec {
        managed.push(ManagedBuffer::new_from_bytes(elem.as_slice()));
    }

    managed
}
