multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait PotlockSetup:
    crate::potlock::PotlockStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(changeFeeForPots)]
    fn change_fee_for_pots(&self, token_identifier: TokenIdentifier, fee: BigUint) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "Invalid token provided"
        );
        self.fee_token_identifier().set(&token_identifier);
        self.fee_amount().set(fee);
    }

    //// internal functions
    fn is_valid_potlock_id(&self, potlock_id: PotlockId) -> bool {
        potlock_id >= 1 && potlock_id <= self.potlocks().len()
    }

    fn require_potlock_exists(&self, potlock_id: PotlockId) {
        require(
            self.is_valid_potlock_id(potlock_id) && !self.potlock().item_is_empty(potlock_id),
            "Potlock doesn't exist!",
        )
    }
}
