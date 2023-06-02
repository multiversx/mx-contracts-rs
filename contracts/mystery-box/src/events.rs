multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config::Reward;

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_create_mystery_box_event(
        &self,
        receiver: &ManagedAddress,
        reward: &ManagedVec<Reward<Self::Api>>,
    ) {
        let epoch = self.blockchain().get_block_epoch();
        self.create_mystery_box_event(receiver, epoch, reward)
    }

    fn emit_open_mystery_box_event(&self, reward: &Reward<Self::Api>) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();
        self.open_mystery_box_event(&caller, epoch, reward)
    }

    #[event("create_mystery_box")]
    fn create_mystery_box_event(
        &self,
        #[indexed] receiver: &ManagedAddress,
        #[indexed] epoch: u64,
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
