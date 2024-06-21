use crate::{
    potlock_setup,
    potlock_storage::{self, PotlockId, ProjectId, Status},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ProjectPercentage = MultiValue2<usize, u64>;

#[multiversx_sc::module]
pub trait PotlockAdminInteractions:
    potlock_storage::PotlockStorage
    + multiversx_sc_modules::only_admin::OnlyAdminModule
    + potlock_setup::PotlockSetup
{
    #[only_admin]
    #[endpoint(acceptPot)]
    fn accept_pot(&self, potlock_id: PotlockId) {
        self.require_potlock_exists(potlock_id);
        let fee_amount = self.fee_amount().get();

        self.fee_amount_accepted_pots()
            .update(|amount| *amount += fee_amount);
        let mut accepted_potlock = self.potlocks().get(potlock_id);
        accepted_potlock.status = Status::Active;
        self.potlocks().set(potlock_id, &accepted_potlock);
        self.fee_pot_proposer(potlock_id).clear();
    }

    #[only_admin]
    #[endpoint(removePot)]
    fn remove_pot(&self, potlock_id: PotlockId) {
        let pot_proposer = self.fee_pot_proposer(potlock_id).get();
        let fee_pot_payment = EsdtTokenPayment::new(
            self.fee_token_identifier().get(),
            0u64,
            self.fee_amount().get(),
        );

        self.send()
            .direct_non_zero_esdt_payment(&pot_proposer, &fee_pot_payment);
        self.fee_pot_proposer(potlock_id).clear();
        self.potlocks().clear_entry(potlock_id);
    }

    #[only_admin]
    #[endpoint(acceptApplication)]
    fn accept_application(&self, project_id: ProjectId) {
        self.require_project_exists(project_id);
        let mut accepted_projects = self.projects().get(project_id);
        accepted_projects.status = Status::Active;
        self.projects().set(project_id, &accepted_projects);
    }

    #[only_admin]
    #[endpoint(rejectDonation)]
    fn reject_donation(&self, potlock_id: PotlockId, user: ManagedAddress) {
        self.require_potlock_exists(potlock_id);
        let opt_fee_pot_payments = self.pot_donations(potlock_id).get(&user);

        require!(opt_fee_pot_payments.is_some(), "No donation for this user");
        let fee_pot_payments = unsafe { opt_fee_pot_payments.unwrap_unchecked() };

        self.send()
            .direct_non_zero_esdt_payment(&user, &fee_pot_payments);
        self.pot_donations(potlock_id).remove(&user);
    }

    #[only_admin]
    #[endpoint(distributePotToProjects)]
    fn distribute_pot_to_projects(
        &self,
        potlock_id: PotlockId,
        project_percentage: MultiValueEncoded<ProjectPercentage>,
    ) {
        self.require_potlock_exists(potlock_id);
        let pot_donations = self.pot_donations(potlock_id);

        for pp in project_percentage {
            let (project_id, percentage) = pp.into_tuple();
            let mut output_payments = ManagedVec::new();
            for (_, donation) in pot_donations.iter() {
                let project_share_amount = donation.amount * percentage;
                let project_share = EsdtTokenPayment::new(
                    donation.token_identifier,
                    donation.token_nonce,
                    project_share_amount,
                );
                output_payments.push(project_share);
            }
            let project_owner = self.projects().get(project_id).owner;
            self.send().direct_multi(&project_owner, &output_payments);
        }

        self.pot_donations(potlock_id).clear();

        //TODO: Clear all info regarding the pot?
    }
}
