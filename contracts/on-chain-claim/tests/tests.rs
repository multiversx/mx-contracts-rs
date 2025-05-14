use config::ConfigModule;
use multiversx_sc::types::{BigUint, EsdtTokenPayment, ManagedAddress, TokenIdentifier};
use multiversx_sc_modules::only_admin::OnlyAdminModule;
use multiversx_sc_scenario::{scenario_model::*, *};
use on_chain_claim::*;

const ON_CHAIN_CLAIM_PATH_EXPR: &str = "mxsc:output/on-chain-claim.mxsc.json";
const TOKEN_IDENTIFIER: &str = "XREPAIR-abcdef";
const OTHER_TOKEN_IDENTIFIER_EXPR: &str = "str:XREPAIRRR-abcdef";
const TOKEN_IDENTIFIER_EXPR: &str = "str:XREPAIR-abcdef";
const TOKEN_NONCE: u64 = 1;
const USER1_ADDR: &str = "address:user1";
const OWNER_ADDR: &str = "address:owner";
const SC_ADDR: &str = "sc:on-chain-claim";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "mxsc:output/on-chain-claim.mxsc.json",
        on_chain_claim::ContractBuilder,
    );
    blockchain
}

#[test]
fn check_token_identifier() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let payment = EsdtTokenPayment::new(
                    TokenIdentifier::from(TOKEN_IDENTIFIER),
                    1u64,
                    BigUint::from(1u64),
                );
                sc.repair_streak_payment().set(payment);
            },
        );
}

#[test]
fn check_before_claim() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        });
}

#[test]
fn check_update_state() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let address = AddressValue::from(OWNER_ADDR).to_address();
                let managed_address = ManagedAddress::from(address);

                sc.add_admin(managed_address);
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(OWNER_ADDR),
            |sc| {
                let address = AddressValue::from(USER1_ADDR).to_address();
                let managed_address = &ManagedAddress::from(address);
                sc.update_state(1u16, managed_address, 5u64, 21u64, 7u64, 5u64);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 5);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.total_epochs_claimed, 7);
        });
}

#[test]
fn check_after_claim() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
        })
        .set_state_step(SetStateStep::new().block_epoch(25))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 25);
        });
}

#[test]
fn check_claim_and_repair() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new()
                        .nonce(1)
                        .esdt_nft_balance(
                            TOKEN_IDENTIFIER_EXPR,
                            TOKEN_NONCE,
                            "100",
                            Option::Some(()),
                        )
                        .esdt_nft_balance(
                            OTHER_TOKEN_IDENTIFIER_EXPR,
                            TOKEN_NONCE,
                            "1",
                            Option::Some(()),
                        ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let payment = EsdtTokenPayment::new(
                    TokenIdentifier::from(TOKEN_IDENTIFIER),
                    1u64,
                    BigUint::from(1u64),
                );
                sc.repair_streak_payment().set(payment);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        })
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("can't repair streak for address");
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info_mapper = sc.address_info(managed_address);

            assert!(address_info_mapper.is_empty());
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, TOKEN_NONCE, "1")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("can't repair streak for current epoch");
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();

            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .set_state_step(SetStateStep::new().block_epoch(22))
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(OTHER_TOKEN_IDENTIFIER_EXPR, 1, "1")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("Bad payment token/amount");
            },
        )
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, TOKEN_NONCE, "2")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("Bad payment token/amount");
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1"),
            |sc| {
                sc.claim_and_repair();
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 22);
            assert_eq!(address_info.best_streak, 3);
        })
        .set_state_step(SetStateStep::new().block_epoch(28))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1"),
            |sc| {
                sc.claim_and_repair();
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 9);
            assert_eq!(address_info.total_epochs_claimed, 9);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 9);
        })
        .set_state_step(SetStateStep::new().block_epoch(36))
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, TOKEN_NONCE, "1")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("can't repair streak for current epoch");
            },
        );
}

#[test]
fn test_best_streak() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let payment = EsdtTokenPayment::new(
                    TokenIdentifier::from(TOKEN_IDENTIFIER),
                    1u64,
                    BigUint::from(1u64),
                );
                sc.repair_streak_payment().set(payment);
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let address = AddressValue::from(OWNER_ADDR).to_address();
                let managed_address = ManagedAddress::from(address);

                sc.add_admin(managed_address);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let repair_streak_payment = sc.repair_streak_payment().get();
            let identifier = TokenIdentifier::from(TOKEN_IDENTIFIER);
            assert_eq!(repair_streak_payment.token_identifier, identifier);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
            assert_eq!(address_info.best_streak, 1);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 2);
        })
        .set_state_step(SetStateStep::new().block_epoch(25))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 25);
            assert_eq!(address_info.best_streak, 2);
        })
        .set_state_step(SetStateStep::new().block_epoch(26))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 4);
            assert_eq!(address_info.last_epoch_claimed, 26);
            assert_eq!(address_info.best_streak, 2);
        })
        .set_state_step(SetStateStep::new().block_epoch(27))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 5);
            assert_eq!(address_info.last_epoch_claimed, 27);
            assert_eq!(address_info.best_streak, 3);
        });
}

#[test]
fn on_chain_claim_whitebox() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let payment = EsdtTokenPayment::new(
                    TokenIdentifier::from(TOKEN_IDENTIFIER),
                    1u64,
                    BigUint::from(1u64),
                );
                sc.repair_streak_payment().set(payment);
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let address = AddressValue::from(OWNER_ADDR).to_address();
                let managed_address = ManagedAddress::from(address);

                sc.add_admin(managed_address);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let repair_streak_payment = sc.repair_streak_payment().get();
            let identifier = TokenIdentifier::from(TOKEN_IDENTIFIER);
            assert_eq!(repair_streak_payment.token_identifier, identifier);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(22))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(26))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(27))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(28))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .whitebox_call_check(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1")
                .no_expect(),
            |sc| {
                sc.claim_and_repair();
            },
            |r| {
                r.assert_user_error("can't repair streak for current epoch");
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 1);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(OWNER_ADDR),
            |sc| {
                let address = AddressValue::from(USER1_ADDR).to_address();
                let managed_address = &ManagedAddress::from(address);
                sc.update_state(1u16, managed_address, 5u64, 21u64, 7u64, 5u64);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 5);
            assert_eq!(address_info.total_epochs_claimed, 7);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 5);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1"),
            |sc| {
                sc.claim_and_repair();
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 12);
            assert_eq!(address_info.total_epochs_claimed, 14);
            assert_eq!(address_info.last_epoch_claimed, 28);
            assert_eq!(address_info.best_streak, 12);
        });
}

#[test]
fn on_chain_claim_seasons_whitebox() {
    let mut world = world();
    let on_chain_claim_whitebox = WhiteboxContract::new(SC_ADDR, on_chain_claim::contract_obj);
    let on_chain_claim_code = world.code_expression(ON_CHAIN_CLAIM_PATH_EXPR);

    let roles: Vec<String> = vec!["ESDTRoleNFTBurn".to_string()];

    world
        .set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDR, Account::new().nonce(1))
                .put_account(
                    USER1_ADDR,
                    Account::new().nonce(1).esdt_nft_balance(
                        TOKEN_IDENTIFIER_EXPR,
                        TOKEN_NONCE,
                        "1",
                        Option::Some(()),
                    ),
                )
                .put_account(
                    SC_ADDR,
                    Account::new()
                        .nonce(1)
                        .code(&on_chain_claim_code)
                        .owner(OWNER_ADDR)
                        .esdt_roles(TOKEN_IDENTIFIER_EXPR, roles),
                )
                .block_epoch(20)
                .new_address(OWNER_ADDR, TOKEN_NONCE, SC_ADDR),
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let payment = EsdtTokenPayment::new(
                    TokenIdentifier::from(TOKEN_IDENTIFIER),
                    1u64,
                    BigUint::from(1u64),
                );
                sc.repair_streak_payment().set(payment);
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(SC_ADDR),
            |sc| {
                let address = AddressValue::from(OWNER_ADDR).to_address();
                let managed_address = ManagedAddress::from(address);

                sc.add_admin(managed_address);
            },
        )
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(OWNER_ADDR),
            |sc| sc.add_season(0u64),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let seasons = sc.seasons().get();
            assert_eq!(seasons.len(), 1);
            assert_eq!(seasons.get(0).id, 1);
            assert_eq!(seasons.get(0).start_epoch, 0);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let repair_streak_payment = sc.repair_streak_payment().get();
            let identifier = TokenIdentifier::from(TOKEN_IDENTIFIER);
            assert_eq!(repair_streak_payment.token_identifier, identifier);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 1u16);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(OWNER_ADDR),
            |sc| sc.add_season(21),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let current_season = sc.get_current_season();
            assert_eq!(current_season.id, 1);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let current_season = sc.get_current_season();
            assert_eq!(current_season.id, 2);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(!can_be_repaired);
        })
        .set_state_step(SetStateStep::new().block_epoch(21))
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(USER1_ADDR),
            |sc| sc.claim(),
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.address_info(managed_address).get();
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 1u16);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 2u16);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 21);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info(managed_address);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 21);
        })
        .set_state_step(SetStateStep::new().block_epoch(23))
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let can_be_repaired = sc.can_be_repaired(managed_address);
            assert!(can_be_repaired);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new()
                .from(USER1_ADDR)
                .to(SC_ADDR)
                .esdt_transfer(TOKEN_IDENTIFIER_EXPR, 1, "1"),
            |sc| {
                sc.claim_and_repair();
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let current_season = sc.get_current_season();
            assert_eq!(current_season.id, 2);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 1u16);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 2u16);
            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 23);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info(managed_address);
            assert_eq!(address_info.current_streak, 3);
            assert_eq!(address_info.total_epochs_claimed, 3);
            assert_eq!(address_info.last_epoch_claimed, 23);
            assert_eq!(address_info.best_streak, 3);
        })
        .whitebox_call(
            &on_chain_claim_whitebox,
            ScCallStep::new().from(OWNER_ADDR),
            |sc| {
                let address = AddressValue::from(USER1_ADDR).to_address();
                let managed_address = &ManagedAddress::from(address);
                sc.update_state(2u16, managed_address, 2u64, 21u64, 2u64, 2u64);
            },
        )
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 1u16);
            assert_eq!(address_info.current_streak, 1);
            assert_eq!(address_info.total_epochs_claimed, 1);
            assert_eq!(address_info.last_epoch_claimed, 20);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info_by_season(managed_address, 2u16);
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 2);
        })
        .whitebox_query(&on_chain_claim_whitebox, |sc| {
            let address = AddressValue::from(USER1_ADDR).to_address();
            let managed_address = &ManagedAddress::from(address);
            let address_info = sc.get_address_info(managed_address);
            assert_eq!(address_info.current_streak, 2);
            assert_eq!(address_info.total_epochs_claimed, 2);
            assert_eq!(address_info.last_epoch_claimed, 21);
            assert_eq!(address_info.best_streak, 2);
        });
}
