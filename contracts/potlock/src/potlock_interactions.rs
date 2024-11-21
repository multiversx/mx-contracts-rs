use __wasm__endpoints__::fee_payment;

use crate::potlock_requirements;
use crate::potlock_storage::{self, Pot, Project};
use crate::potlock_storage::{PotlockId, ProjectId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait PotlockInteractions:
    potlock_requirements::PotlockRequirements
    + potlock_storage::PotlockStorage
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[payable("*")]
    #[endpoint(addPot)]
    fn add_pot(&self, name: ManagedBuffer, description: ManagedBuffer) {
        let fee_payment = self.call_value().single_esdt();

        require!(
            fee_payment().get() == fee_payment,
            "Wrong payment for creating a pot!"
        );
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
    ) -> usize {
        let owner = self.blockchain().get_caller();
        let project = Project::new(potlock_id, project_name, description, owner);
        self.projects().push(&project)
    }

    #[payable("*")]
    #[endpoint(donateToPot)]
    fn donate_to_pot(&self, potlock_id: PotlockId) {
        self.require_potlock_exists(potlock_id);
        self.require_potlock_is_active(potlock_id);

        let (payment_token_id, payment_amount) = self.call_value().single_fungible_esdt();
        let caller = self.blockchain().get_caller();
        let mut donation_mapper = self.pot_donations(potlock_id);

        if donation_mapper.contains_key(&caller) {
            let opt_payment = donation_mapper.get(&caller);
            if opt_payment.is_some() {
                let mut previous_payment = opt_payment.unwrap();
                require!(
                    previous_payment.token_identifier == payment_token_id.clone(),
                    "Already made a payment with a different TokenID"
                );
                previous_payment.amount += payment_amount;
                donation_mapper.insert(caller, previous_payment);
            }
        } else {
            donation_mapper.insert(
                caller,
                EsdtTokenPayment::new(payment_token_id, 0, payment_amount),
            );
        }
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
            let opt_payment = donation_mapper.get(&caller);
            if opt_payment.is_some() {
                let mut previous_payment = opt_payment.unwrap();
                require!(
                    previous_payment.token_identifier == payment.token_identifier.clone(),
                    "Already made a payment with a different TokenID"
                );
                previous_payment.amount += payment.amount;
                donation_mapper.insert(caller, previous_payment);
            }
        } else {
            donation_mapper.insert(caller, payment);
        }
    }
}
