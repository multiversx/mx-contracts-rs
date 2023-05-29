mod mystery_box_setup;
use mystery_box_setup::*;

#[test]
fn test_mystery_box_setup() {
    let _ = MysteryBoxSetup::new(mystery_box::contract_obj);
}

#[test]
fn test_create_and_open_mystery_box() {
    let mut mb_setup = MysteryBoxSetup::new(mystery_box::contract_obj);
    let mb_token_nonce =
        mb_setup.create_mystery_box(900, 4_999, 1, 4_000, 1_500, 1_000, 50, 1, 1, 1);
    mb_setup.open_mystery_box(mb_token_nonce);
    let mb_token_nonce =
        mb_setup.create_mystery_box(900, 4_999, 1, 4_000, 1_500, 1_000, 50, 1, 2, 1);
    mb_setup.open_mystery_box(mb_token_nonce);
}
