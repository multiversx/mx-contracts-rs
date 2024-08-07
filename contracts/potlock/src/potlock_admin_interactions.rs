use crate::potlock_storage::{self, PotlockId, ProjectId, Status};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ProjectPercentage = MultiValue2<usize, u64>;
pub const MAX_PERCENTAGE: u64 = 10_000; // 100%

#[multiversx_sc::module]
pub trait PotlockAdminInteractions:
    potlock_storage::PotlockStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(changeFeeForPots)]
    fn change_fee_for_pots(&self, token_identifier: TokenIdentifier, fee: BigUint) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "Invalid token provided"
        );
        require!(
            token_identifier.is_valid_esdt_identifier() && fee.ge(&BigUint::zero()),
            "Invalid token identifier or amount is 0"
        );
        self.fee_token_identifier().set(&token_identifier);
        self.fee_amount().set(fee);
    }

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
    }

    #[only_admin]
    #[endpoint(removePot)]
    fn remove_pot(&self, potlock_id: PotlockId) {
        self.require_potlock_exists(potlock_id);

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
        project_percentages: MultiValueEncoded<ProjectPercentage>,
    ) {
        self.require_potlock_exists(potlock_id);
        self.require_correct_percentages(project_percentages.clone());
        let pot_donations = self.pot_donations(potlock_id);

        for pp in project_percentages {
            let (project_id, percentage) = pp.into_tuple();
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
