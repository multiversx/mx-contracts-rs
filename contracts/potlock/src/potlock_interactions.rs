multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait PotlockInteractions: crate::potlock::PotlockStorage {
    #[payable("*")]
    #[endpoint(addPot)]
    fn add_pot(&self, name: ManagedBuffer, description: ManagedBuffer) {
        let payment = self.call_value().egld_or_single_esdt().into_tuple();
        require!(
            self.fee_token_identifier().get() == payment.token_identifier,
            "Wrong token identifier for creating a pot!"
        );
        require!(
            self.fee_amount().get() == payment.amount,
            "Wrong fee amount for creating a pot"
        );
        let caller = self.blockchain().get_caller();

        let potlock = Potlock::new(name, description);
        let potlock_id = self.potlocks().push(&potlock);
        let fee_pot_payments = self.fee_pot_payments(potlock_id, caller).insert(payment);
    }

    #[endpoint(applyForPot)]
    fn apply_for_pot(&self, project_name: ManagedBuffer, description: ManagedBuffer) {
        // TODO: should we set a SC address as a parameter or assume (and verifiy) that the SC will call this endpoint
        // TODO: This address will receive the funds
        let caller = self.blockchain().get_caller();
        let project = Project::new(name, description);
        let project_id = self.projects().push(&project);
    }

    #[payable("*")]
    #[endpoint(donateToPot)]
    fn donate_to_pot(&self, potlock_id: PotlockId) {
        let payment = self.call_value().egld_or_single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        self.fee_pot_payments(potlock_id).insert(payment);
    }

    #[payable("*")]
    #[endpoint(donateToProject)]
    fn donate_to_project(&self, project_id: ProjectId) {
        let payment = self.call_value().egld_or_single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        self.fee_project_payments(project_id, caller)
            .insert(payment);
    }
}
