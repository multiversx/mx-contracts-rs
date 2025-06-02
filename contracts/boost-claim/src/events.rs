#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_boost_claim_event(&self, caller: &ManagedAddress, prize: &ManagedBuffer) {
        self.boost_claim_event(caller, prize)
    }

    #[event("boost_claim")]
    fn boost_claim_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] prize: &ManagedBuffer,
    );
}
