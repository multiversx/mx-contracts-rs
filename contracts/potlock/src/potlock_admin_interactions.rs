multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ProjectPercentage = MultiValue2<usize, usize>;

#[multiversx_sc::module]
pub trait PotlockAdminInteractions:
    crate::potlock::PotlockStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(acceptPot)]
    fn accept_pot(&self, potlock_id: PotlockId) {
        require_potlock_exists(potlock_id);
        // TODO: Common fund is another contract?
    }

    #[only_admin]
    #[endpoint(rejectPot)]
    fn reject_pot(&self, potlock_id: PotlockId) {
        require_potlock_exists(potlock_id);

        //TODO: Common fund is another contract?
        //TODO: "will return the fee back to the user"

        //TODO: Should we remove the potlock?
        self.potlocks().clear_entry(proposal_id);
    }

    #[only_admin]
    #[endpoint(removePot)]
    fn remove_pot(&self, potlock_id: PotlockId) {
        let caller = self.blockchain().get_caller();
        let payment = self.fee_pot_payments(potlock_id, caller).get();

        self.send().direct_non_zero_esdt_payment(&caller, &payment);
        self.potlocks().clear_entry(proposal_id);
    }

    #[only_admin]
    #[endpoint(acceptApplication)]
    fn accept_application(&self, project: Projectid) {
        require_potlock_exists(potlock_id);
        // TODO: How should we KYC verification in the SC?
    }
    rejectDonation@userID@listOfTokens - returns tokens to the users.

    #[only_admin]
    #[endpoint(rejectDonation)]
    fn reject_donation(&self, potlock: PotlockId, user: ManagedAddress) {
        require_potlock_exists(potlock_id);
        let fee_pot_payments = self.fee_pot_payments(potlock_id, user);
        self.send().direct_non_zero_esdt_payment(&user, &fee_pot_payments);
    }


    #[only_admin]
    #[endpoint(distributePotToProjects)]
    fn distribute_pot_to_projects(&self, potlock: PotlockId, project_percentage: MultiValueEncoded<ProjectPercentage>) {
        require_potlock_exists(potlock_id);
        let potlock = self.potlocks().get();
        for pp in project_percentage {
            let (project_id, percentage) = pp.into_tuple();

        }
        // TODO: How should we KYC verification in the SC?
    }

    fn get_total_payments_for_pot(&self) {
        let fee_pot_payments = self.fee_pot_payments().get();   
    }
}
