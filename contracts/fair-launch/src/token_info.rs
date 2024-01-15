multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait TokenInfoModule {
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        token_type: EsdtTokenType,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        require!(token_type != EsdtTokenType::Invalid, "Invalid token type");

        let payment_amount = self.call_value().egld_value().clone_value();
        match token_type {
            EsdtTokenType::Fungible => {
                self.fungible_token().issue_and_set_all_roles(
                    payment_amount,
                    token_display_name,
                    token_ticker,
                    num_decimals,
                    None,
                );
            }
            _ => self.non_fungible_token().issue_and_set_all_roles(
                token_type,
                payment_amount,
                token_display_name,
                token_ticker,
                num_decimals,
                None,
            ),
        }
    }

    #[only_owner]
    #[endpoint(setTransferRole)]
    fn set_transfer_role(&self) {
        self.non_fungible_token()
            .set_local_roles(&[EsdtLocalRole::Transfer], None);
    }

    #[storage_mapper("tokenId")]
    fn fungible_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[storage_mapper("tokenId")]
    fn non_fungible_token(&self) -> NonFungibleTokenMapper<Self::Api>;
}
