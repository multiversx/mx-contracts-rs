use crate::potlock_storage::{self, PotlockId, ProjectId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PotlockSetup:
    potlock_storage::PotlockStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
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
        require!(
            self.is_valid_potlock_id(potlock_id) && !self.potlocks().item_is_empty(potlock_id),
            "Potlock doesn't exist!",
        )
    }

    fn is_valid_project_id(&self, project_id: ProjectId) -> bool {
        project_id >= 1 && project_id <= self.projects().len()
    }

    fn require_project_exists(&self, project_id: ProjectId) {
        require!(
            self.is_valid_project_id(project_id) && !self.projects().item_is_empty(project_id),
            "Project doesn't exist!",
        )
    }
}
