use super::common::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SimpleTransferModule: super::common::CommonModule {
    // TODO: Use the new API which returns the tokens if it fails, when it's added to the framework

    fn perform_simple_transfer_egld(&self, to: &ManagedAddress, egld_value: &BigUint) {
        self.require_dest_not_self(to);

        self.send().direct_egld(to, egld_value);
    }

    fn perform_simple_transfer_esdt(
        &self,
        to: &ManagedAddress,
        esdt_payments: &PaymentsVec<Self::Api>,
    ) {
        self.require_dest_not_self(to);
        self.require_not_empty_payments(esdt_payments);

        self.send().direct_multi(to, esdt_payments);
    }
}
