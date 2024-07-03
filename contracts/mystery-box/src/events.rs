use multiversx_sc::imports::*;

use crate::config::Reward;

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_create_mystery_box_event(
        &self,
        caller: &ManagedAddress,
        current_epoch: u64,
        payment: &EsdtTokenPayment,
        rewards: &ManagedVec<Reward<Self::Api>>,
    ) {
        self.create_mystery_box_event(caller, current_epoch, payment, rewards);
    }

    fn emit_open_mystery_box_event(
        &self,
        caller: &ManagedAddress,
        current_epoch: u64,
        reward: &Reward<Self::Api>,
    ) {
        self.open_mystery_box_event(caller, current_epoch, reward)
    }

    #[event("create_mystery_box")]
    fn create_mystery_box_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] current_epoch: u64,
        #[indexed] payment: &EsdtTokenPayment,
        reward: &ManagedVec<Reward<Self::Api>>,
    );

    #[event("open_mystery_box")]
    fn open_mystery_box_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] epoch: u64,
        reward: &Reward<Self::Api>,
    );
}
