multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SimpleTransferModule: super::common::CommonModule {
    fn perform_simple_transfer_egld(&self) {
        // self.require_dest_not_self();

        // TODO
    }

    fn perform_simple_transfer_esdt(&self) {
        // self.require_dest_not_self();

        // TODO
    }
}
