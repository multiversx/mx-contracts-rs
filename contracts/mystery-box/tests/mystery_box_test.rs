mod mystery_box_setup;
use multiversx_sc_scenario::rust_biguint;
use mystery_box_setup::*;

#[test]
fn test_mystery_box_setup() {
    let _ = MysteryBoxSetup::new(mystery_box::contract_obj);
}

#[test]
fn test_create_and_open_mystery_box() {
    let mut mb_setup = MysteryBoxSetup::new(mystery_box::contract_obj);

    let fixed_value_reward_amount = 50u64;
    mb_setup.b_mock.set_block_epoch(1);
    let mb_token_nonce = mb_setup.create_mystery_box(
        900,
        5_999,
        1,
        1,
        3_000,
        0,
        1_500,
        1_000,
        0,
        fixed_value_reward_amount,
        1,
        1,
        1,
        1,
    );
    mb_setup.open_mystery_box(mb_token_nonce);

    // We need to change the block random seed to properly test the RandomnessSource functionality
    mb_setup.b_mock.set_block_random_seed(Box::from([1u8; 48]));
    let mb_token_nonce = mb_setup.create_mystery_box(
        900,
        5_999,
        1,
        1,
        3_000,
        0,
        1_500,
        1_000,
        0,
        fixed_value_reward_amount,
        1,
        1,
        2,
        1,
    );
    mb_setup.open_mystery_box(mb_token_nonce);

    mb_setup.b_mock.set_block_epoch(2);
    mb_setup.b_mock.set_block_random_seed(Box::from([2u8; 48]));
    let mb_token_nonce = mb_setup.create_mystery_box(
        900,
        5_999,
        1,
        1,
        3_000,
        0,
        1_500,
        1_000,
        0,
        fixed_value_reward_amount,
        1,
        1,
        3,
        1,
    );
    mb_setup.open_mystery_box(mb_token_nonce);

    // Fixed value high chance
    mb_setup
        .b_mock
        .check_egld_balance(&mb_setup.user_address, &rust_biguint!(0u64));

    mb_setup.b_mock.set_block_epoch(3);
    mb_setup.b_mock.set_block_random_seed(Box::from([3u8; 48]));
    mb_setup.deposit_rewards(fixed_value_reward_amount);
    let mb_token_nonce = mb_setup.create_mystery_box(
        900,
        1,
        1,
        1,
        1,
        0,
        1_500,
        1,
        0,
        fixed_value_reward_amount,
        9_997,
        1,
        4,
        1,
    );
    mb_setup.open_mystery_box(mb_token_nonce);
    mb_setup.b_mock.check_egld_balance(
        &mb_setup.user_address,
        &rust_biguint!(fixed_value_reward_amount),
    );
}
