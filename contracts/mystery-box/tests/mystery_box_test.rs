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

    mb_setup.setup_mystery_box();
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
    let mb_token_nonce = mb_setup.create_mystery_box(10);

    // We change the block random seed to properly test the RandomnessSource functionality
    mb_setup.b_mock.set_block_epoch(3);
    mb_setup.b_mock.set_block_random_seed(Box::from([2u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);

    // Change the block random seed
    mb_setup.b_mock.set_block_random_seed(Box::from([3u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);

    // Keep the block random seed the same for 3 mystery boxes (ResetOnCooldown)
    // This should select the same reward, which allows 2 wins per cooldown
    mb_setup.b_mock.set_block_epoch(4);
    mb_setup.b_mock.set_block_random_seed(Box::from([4u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);
    mb_setup.open_mystery_box(mb_token_nonce);

    // Now a new prize is selected, as the initial one is on cooldown
    // A lifetime prize is opened, so it should not be won again from now on
    mb_setup.open_mystery_box(mb_token_nonce);

    // The 2 epochs cooldown has passed, ResetOnCooldown prize can be won again
    mb_setup.b_mock.set_block_epoch(6);
    mb_setup.b_mock.set_block_random_seed(Box::from([6u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);
    mb_setup.open_mystery_box(mb_token_nonce);

    // The ResetOnCooldown prize cannot be won the third time during the cooldown, so a new one is selected
    mb_setup.open_mystery_box(mb_token_nonce);

    // The lifetime reward is selected again, but it can't be won again, so a new prize is selected
    mb_setup.b_mock.set_block_epoch(7);
    mb_setup.b_mock.set_block_random_seed(Box::from([7u8; 48]));
    mb_setup.open_mystery_box(mb_token_nonce);
}
