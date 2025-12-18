use crate::{
    potlock_requirements::{self, MAX_PERCENTAGE},
    potlock_storage::{self, PotlockId, ProjectId, Status},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ProjectPercentage = MultiValue2<usize, u64>;

#[multiversx_sc::module]
pub trait PotlockAdminInteractions:
    potlock_requirements::PotlockRequirements
    + potlock_storage::PotlockStorage
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(changeFeeForPots)]
    fn change_fee_for_pots(&self, token_identifier: TokenIdentifier, fee: BigUint) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "Invalid token provided"
        );
        require!(fee > 0, "Amount is 0");
        self.fee_token_identifier().set_if_empty(&token_identifier);
        self.fee_amount().set_if_empty(fee);
    }

    #[only_admin]
    #[endpoint(acceptPot)]
    fn accept_pot(&self, potlock_id: PotlockId) {
        self.require_potlock_exists(potlock_id);
        self.require_potlock_is_inactive(potlock_id);

        let mut accepted_potlock = self.potlocks().get(potlock_id);
        accepted_potlock.status = Status::Active;
        self.potlocks().set(potlock_id, &accepted_potlock);
    }

    #[only_admin]
    #[endpoint(removePot)]
    fn remove_pot(&self, potlock_id: PotlockId) {
        self.require_potlock_exists(potlock_id);
        self.require_potlock_is_inactive(potlock_id);

        let potlock_mapper = self.potlocks();
        let pot_proposer = potlock_mapper.get(potlock_id).proposer;
        let fee_pot_payment = EsdtTokenPayment::new(
            self.fee_token_identifier().get(),
            0u64,
            self.fee_amount().get(),
        );

        self.send()
            .direct_non_zero_esdt_payment(&pot_proposer, &fee_pot_payment);
        self.potlocks().clear_entry(potlock_id);
    }

    #[only_admin]
    #[endpoint(acceptApplication)]
    fn accept_application(&self, project_id: ProjectId) {
        self.require_project_exists(project_id);
        self.require_project_is_inactive(project_id);

        let mut accepted_project = self.projects().get(project_id);
        accepted_project.status = Status::Active;
        self.projects().set(project_id, &accepted_project);
    }

    #[only_admin]
    #[endpoint(removeApplication)]
    fn remove_application(&self, project_id: ProjectId) {
        self.require_project_exists(project_id);
        self.require_project_is_active(project_id);

        let mut rejected_project = self.projects().get(project_id);
        rejected_project.status = Status::Inactive;
        self.projects().set(project_id, &rejected_project);
    }

    #[only_admin]
    #[endpoint(rejectDonation)]
    fn reject_donation(&self, potlock_id: PotlockId, user: ManagedAddress) {
        self.require_potlock_exists(potlock_id);
        let opt_fee_pot_payment = self.pot_donations(potlock_id).get(&user);

        require!(opt_fee_pot_payment.is_some(), "No donation for this user");
        let fee_pot_payment = unsafe { opt_fee_pot_payment.unwrap_unchecked() };

        self.send()
            .direct_non_zero_esdt_payment(&user, &fee_pot_payment);
        self.pot_donations(potlock_id).remove(&user);
    }

    #[only_admin]
    #[endpoint(distributePotToProjects)]
    fn distribute_pot_to_projects(
        &self,
        potlock_id: PotlockId,
        project_percentages: MultiValueEncoded<ProjectPercentage>,
    ) {
        self.require_potlock_exists(potlock_id);
        self.require_correct_percentages(project_percentages.clone());
        let pot_donations = self.pot_donations(potlock_id);
        let all_projects = self.projects();

        for pp in project_percentages {
            let (project_id, percentage) = pp.into_tuple();
            let project = all_projects.get(project_id);

            // The project must previously apply to this Pot
            if project.potlock_id != potlock_id {
                continue;
            }

            let mut output_payments = ManagedVec::new();
            for (_, donation) in pot_donations.iter() {
                let project_share_amount = donation.amount * percentage / MAX_PERCENTAGE;
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
    }
}
