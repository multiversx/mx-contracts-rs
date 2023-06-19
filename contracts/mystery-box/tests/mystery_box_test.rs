mod mystery_box_setup;
use mystery_box_setup::*;

#[test]
fn test_mystery_box_setup() {
    let _ = MysteryBoxSetup::new(mystery_box::contract_obj);
}

#[test]
fn test_mb_token_nonce_stacking() {
    let mut mb_setup = MysteryBoxSetup::new(mystery_box::contract_obj);

    mb_setup.b_mock.set_block_epoch(1);
    let first_mb_token_nonce = mb_setup.create_mystery_box(1);
    let second_mb_token_nonce = mb_setup.create_mystery_box(1);

    assert_eq!(first_mb_token_nonce, second_mb_token_nonce);

    mb_setup.b_mock.set_block_epoch(2);
    mb_setup.open_mystery_box(second_mb_token_nonce);

    mb_setup.setup_mystery_box(
        1_500, 5_999, 1, 3_000, 0, 1_500, 1_000, 0, 50, 1, 1, 0, 0, 0,
    );
    let third_mb_token_nonce = mb_setup.create_mystery_box(1);

    // Should be different, as they were with different attributes
    assert_ne!(first_mb_token_nonce, third_mb_token_nonce);

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

    mb_setup.b_mock.set_block_epoch(4);
    mb_setup.b_mock.set_block_random_seed(Box::from([4u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);
    mb_setup.open_mystery_box(mb_token_nonce);
}
