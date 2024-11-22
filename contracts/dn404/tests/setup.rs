use dn404::{dn404_proxy, Dn404, Percentage};
use multiversx_sc::types::{
    EsdtLocalRole, MultiValueEncoded, TestAddress, TestEsdtTransfer, TestSCAddress,
    TestTokenIdentifier,
};
use multiversx_sc_scenario::{imports::MxscPath, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld};

pub const FRACTAL_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FRACTAL-123456");
pub const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
pub const PRICE_NONCE_2: u64 = 50;
pub const PRICE_COL: u64 = 25;
pub const FEE_NONCE_2: u64 = 10;
pub const FEE_COL: u64 = 5;
pub const FEE_BASKET: Percentage = 1_000; // 10%
pub const USER_BALANCE: u64 = 1_000;
pub const OWNER: TestAddress = TestAddress::new("owner");
pub const FIRST_USER: TestAddress = TestAddress::new("first");
pub const SECOND_USER: TestAddress = TestAddress::new("second");
pub const DN404_ADDRESS: TestSCAddress = TestSCAddress::new("dn404");
pub const CODE_PATH: MxscPath = MxscPath::new("output/dn404.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("contracts/dn404");
    blockchain.register_contract(CODE_PATH, dn404::ContractBuilder);
    blockchain
}

pub struct Dn404Setup {
    pub b_mock: ScenarioWorld,
}

impl Default for Dn404Setup {
    fn default() -> Self {
        Self::new()
    }
}

impl Dn404Setup {
    pub fn new() -> Self {
        let mut b_mock = world();

        let roles = vec![
            EsdtLocalRole::Mint.name().to_string(),
            EsdtLocalRole::Burn.name().to_string(),
        ];

        b_mock
            .account(OWNER)
            .nonce(1)
            .esdt_nft_balance(NFT_TOKEN_ID, 5, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 6, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 7, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 8, 1, ());

        b_mock
            .account(DN404_ADDRESS)
            .nonce(1)
            .code(CODE_PATH)
            .owner(OWNER)
            .esdt_roles(FRACTAL_TOKEN_ID, roles);

        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .whitebox(dn404::contract_obj, |sc| {
                sc.init(
                    FRACTAL_TOKEN_ID.to_token_identifier(),
                    MultiValueEncoded::new(),
                );
            });

        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .set_internal_price_for_token(NFT_TOKEN_ID, 2u64, PRICE_NONCE_2)
            .run();
        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .set_internal_price_for_collection(NFT_TOKEN_ID, PRICE_COL)
            .run();
        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .set_fee_for_fractionalizing_nft(NFT_TOKEN_ID, 2u64, FEE_NONCE_2)
            .run();
        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .set_fee_for_fractionalizing_collection(NFT_TOKEN_ID, FEE_COL)
            .run();
        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .set_fee_for_deposit_basket_of_goods(FEE_BASKET)
            .run();
        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .unpause_endpoint()
            .run();

        let transfers = vec![
            TestEsdtTransfer(NFT_TOKEN_ID, 5, 1),
            TestEsdtTransfer(NFT_TOKEN_ID, 6, 1),
            TestEsdtTransfer(NFT_TOKEN_ID, 7, 1),
            TestEsdtTransfer(NFT_TOKEN_ID, 8, 1),
        ];

        b_mock
            .tx()
            .from(OWNER)
            .to(DN404_ADDRESS)
            .typed(dn404_proxy::Dn404Proxy)
            .deposit()
            .multi_esdt(transfers)
            .run();

        b_mock
            .account(FIRST_USER)
            .nonce(1)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 2, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 3, 1, ())
            .esdt_nft_balance(NFT_TOKEN_ID, 4, 1, ());

        b_mock
            .account(SECOND_USER)
            .nonce(1)
            .esdt_balance(FRACTAL_TOKEN_ID, USER_BALANCE);

        Dn404Setup { b_mock }
    }
}
