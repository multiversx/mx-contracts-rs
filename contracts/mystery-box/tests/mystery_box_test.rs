mod mystery_box_setup;
use mystery_box_setup::*;

#[test]
fn test_mystery_box_setup() {
    let _ = MysteryBoxSetup::new(mystery_box::contract_obj);
}

#[test]
fn test_mb_token_nonce_stacking_and_opening_cooldown() {
    let mut mb_setup = MysteryBoxSetup::new(mystery_box::contract_obj);

    mb_setup.b_mock.set_block_epoch(1);
    let first_mb_token_nonce = mb_setup.create_mystery_box(1);
    let second_mb_token_nonce = mb_setup.create_mystery_box(1);

    // Should be equal as they are created in the same epoch
    assert_eq!(first_mb_token_nonce, second_mb_token_nonce);

    // Should throw error, as the 1 epoch cooldown has not passed
    mb_setup.open_mystery_box_cooldown_error_expected(first_mb_token_nonce);

    mb_setup.b_mock.set_block_epoch(2);

    // Should be processed succesfully, as the cooldown period has passed
    mb_setup.open_mystery_box(second_mb_token_nonce);

    let third_mb_token_nonce = mb_setup.create_mystery_box(1);

    // Should be different, as they were created in different epochs
    assert_ne!(first_mb_token_nonce, third_mb_token_nonce);
    mb_setup.open_mystery_box_cooldown_error_expected(third_mb_token_nonce);

    mb_setup.b_mock.set_block_epoch(3);
    mb_setup.open_mystery_box(third_mb_token_nonce);
}

#[test]
fn test_open_multiple_mystery_box() {
    let mut mb_setup = MysteryBoxSetup::new(mystery_box::contract_obj);

    mb_setup.b_mock.set_block_epoch(1);
    let mb_token_nonce = mb_setup.create_mystery_box(3);

    // We need to change the block random seed to properly test the RandomnessSource functionality
    mb_setup.b_mock.set_block_epoch(2);
    mb_setup.b_mock.set_block_random_seed(Box::from([2u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);

    // We're still in epoch 2
    // The first chosen reward (ExperiencePoints) is on global cooldown,
    // So a MysteryBox reward is chosen next (which has no cooldown)
    // The user receives directly a new MysteryBox, with a different nonce (new epoch)
    mb_setup.b_mock.set_block_random_seed(Box::from([3u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);

    let new_mb_token_nonce = 2;
    // It should throw an error, as it is still the same epoch from when it was created
    mb_setup.open_mystery_box_cooldown_error_expected(new_mb_token_nonce);

    mb_setup.b_mock.set_block_epoch(4);
    mb_setup.b_mock.set_block_random_seed(Box::from([4u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);
    mb_setup.open_mystery_box(new_mb_token_nonce);
}
