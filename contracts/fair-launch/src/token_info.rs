use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait TokenInfoModule:
    multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
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
        let payment_amount = self.call_value().egld().clone();
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

    fn get_token_id(&self) -> TokenIdentifier {
        self.non_fungible_token().get_token_id()
    }

    // Both storage mappers must use the same key!

    #[storage_mapper("tokenId")]
    fn fungible_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[storage_mapper("tokenId")]
    fn non_fungible_token(&self) -> NonFungibleTokenMapper<Self::Api>;
}
