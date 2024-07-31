use crate::potlock_storage::{self, Pot, Project};
use crate::potlock_storage::{PotlockId, ProjectId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait PotlockInteractions:
    potlock_storage::PotlockStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[payable("*")]
    #[endpoint(addPot)]
    fn add_pot(&self, name: ManagedBuffer, description: ManagedBuffer) {
        let payment_for_adding_pot = self.call_value().single_esdt();
        require!(
            self.fee_token_identifier().get() == payment_for_adding_pot.token_identifier,
            "Wrong token identifier for creating a pot!"
        );
        require!(
            self.fee_amount().get() == payment_for_adding_pot.amount,
            "Wrong fee amount for creating a pot"
        );
        let caller = self.blockchain().get_caller();

        let potlock_id = self.potlocks().len() + 1;
        let potlock = Pot::new(potlock_id, caller, name, description);
        self.potlocks().push(&potlock);
    }

    #[endpoint(applyForPot)]
    fn apply_for_pot(
        &self,
        potlock_id: PotlockId,
        project_name: ManagedBuffer,
        description: ManagedBuffer,
    ) {
        let project_id = self.projects().len() + 1;
        let owner = self.blockchain().get_caller();
        let project = Project::new(project_id, potlock_id, project_name, description, owner);
        self.projects().push(&project);
    }

    #[payable("*")]
    #[endpoint(donateToPot)]
    fn donate_to_pot(&self, potlock_id: PotlockId) {
        let payment = self.call_value().single_esdt();
        let caller = self.blockchain().get_caller();
        self.require_potlock_exists(potlock_id);
        self.require_potlock_is_active(potlock_id);
        self.pot_donations(potlock_id).insert(caller, payment);
    }

    #[payable("*")]
    #[endpoint(donateToProject)]
    fn donate_to_project(&self, project_id: ProjectId) {
        self.require_project_exists(project_id);
        self.require_project_is_active(project_id);
        let payment = self.call_value().single_esdt();
        let caller = self.blockchain().get_caller();

        let mut donation_mapper = self.project_donations(project_id);
        if donation_mapper.contains_key(&caller) {
            let opt_payment = donation_mapper.remove(&caller);
            if opt_payment.is_some() {
                let mut previous_payment = opt_payment.unwrap();
                require!(
                    previous_payment.token_identifier == payment.token_identifier.clone(),
                    "Already made a payment with a different TokenID"
                );
                previous_payment.amount += payment.amount;
                self.project_donations(project_id)
                    .insert(caller, previous_payment);
            }
        } else {
            self.project_donations(project_id).insert(caller, payment);
        }
    }
}
