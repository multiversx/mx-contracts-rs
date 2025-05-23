use crate::{
    call_dispatcher::CallType,
    raw_call::common::{PaymentsVec, RawCall},
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HighLevelCallsModule:
    crate::raw_call::simple_transfer::SimpleTransferModule
    + crate::raw_call::sync_call::SyncCallModule
    + crate::raw_call::async_call::AsyncCallModule
    + crate::raw_call::common::CommonModule
{
    #[must_use]
    fn perform_no_transfer(
        &self,
        dest_address: ManagedAddress,
        call_type: CallType,
        opt_raw_call_args: Option<RawCall<Self::Api>>,
    ) -> Option<BackTransfers<Self::Api>> {
        match call_type {
            CallType::SimpleTransfer => {
                sc_panic!("May not perform simple transfer with no actual transfers")
            }
            CallType::Sync => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                let back_transfers =
                    self.perform_raw_sync_call_egld(dest_address, raw_call_args, BigUint::zero());

                Some(back_transfers)
            }
            CallType::Async => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                self.perform_raw_async_call_egld(dest_address, raw_call_args, BigUint::zero());

                None
            }
        }
    }

    #[must_use]
    fn perform_egld_transfer(
        &self,
        dest_address: ManagedAddress,
        call_type: CallType,
        egld_value: BigUint,
        opt_raw_call_args: Option<RawCall<Self::Api>>,
    ) -> Option<BackTransfers<Self::Api>> {
        match call_type {
            CallType::SimpleTransfer => {
                self.perform_simple_transfer_egld(&dest_address, &egld_value);

                None
            }
            CallType::Sync => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                let back_transfers =
                    self.perform_raw_sync_call_egld(dest_address, raw_call_args, egld_value);

                Some(back_transfers)
            }
            CallType::Async => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                self.perform_raw_async_call_egld(dest_address, raw_call_args, egld_value);

                None
            }
        }
    }

    #[must_use]
    fn perform_multi_esdt_transfer(
        &self,
        dest_address: ManagedAddress,
        call_type: CallType,
        esdt_payments: PaymentsVec<Self::Api>,
        opt_raw_call_args: Option<RawCall<Self::Api>>,
    ) -> Option<BackTransfers<Self::Api>> {
        match call_type {
            CallType::SimpleTransfer => {
                self.perform_simple_transfer_esdt(&dest_address, &esdt_payments);

                None
            }
            CallType::Sync => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                let back_transfers =
                    self.perform_raw_sync_call_esdt(dest_address, raw_call_args, esdt_payments);

                Some(back_transfers)
            }
            CallType::Async => {
                let raw_call_args = self.unwrap_raw_call_args_or_panic(opt_raw_call_args);
                self.perform_raw_async_call_esdt(dest_address, raw_call_args, esdt_payments);

                None
            }
        }
    }

    #[inline]
    #[must_use]
    fn unwrap_raw_call_args_or_panic(
        &self,
        opt_raw_call_args: Option<RawCall<Self::Api>>,
    ) -> RawCall<Self::Api> {
        opt_raw_call_args.unwrap_or_else(|| sc_panic!("No exec args provided"))
    }
}
