use crate::{
    raw_call::common::{FunctionName, GasLimit, PaymentsVec, RawArgs, RawCall},
    unique_payments::UniquePayments,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub enum PaymentType<M: ManagedTypeApi> {
    None,
    Egld { amount: BigUint<M> },
    FixedPayments { esdt_payments: PaymentsVec<M> },
    ReceivedPaymentsFromSc,
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub enum CallType {
    SimpleTransfer,
    Sync,
    Async,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct FunctionNameArgsPair<M: ManagedTypeApi> {
    pub function_name: FunctionName<M>,
    pub args: RawArgs<M>,
}

pub type SingleCallArg<M> = MultiValue5<
    ManagedAddress<M>,
    PaymentType<M>,
    CallType,
    GasLimit,
    Option<FunctionNameArgsPair<M>>,
>;

#[multiversx_sc::module]
pub trait CallDispatcherModule:
    crate::raw_call::simple_transfer::SimpleTransferModule
    + crate::raw_call::sync_call::SyncCallModule
    + crate::raw_call::async_call::AsyncCallModule
    + crate::raw_call::common::CommonModule
    + crate::high_level_calls::HighLevelCallsModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Bundle multiple actions into one tx. You may pass multiple arguments of the following type:
    ///
    /// Args:
    /// - dest_address - The destination address for the transfer/action. Must be a SC if any execution is wanted.
    /// - payment_type - You can choose between no payments, EGLD, or multiple ESDT tokens. You may also choose to use the tokens received from the SC starting from the 2nd argument onwards.
    /// - call_type - The type of call wanted. Currently supports simple transfers, sync calls or async calls.
    /// - gas_limit - Used for execution. Ignored for simple transfers. Must be at least 1M if execution is wanted. Not bundled with execution args to save some encoding space.
    /// - function_name + args - Option type arg (NOT optional!). Pass if you want execution. Gas limit must be at least 1M.
    #[payable("*")]
    #[endpoint(multiCall)]
    fn multi_call(&self, args: MultiValueEncoded<SingleCallArg<Self::Api>>) {
        self.require_not_paused();

        let mut total_egld = self.call_value().egld_direct_non_strict().clone_value();
        let mut all_esdt =
            UniquePayments::new_from_payments(self.call_value().all_esdt_transfers().clone_value());
        let mut opt_last_back_transfers = None;

        for arg in args {
            self.perform_single_call_from_arg(
                arg,
                &mut total_egld,
                &mut all_esdt,
                &mut opt_last_back_transfers,
            );
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_non_zero_egld(&caller, &total_egld);

        let all_esdt_payments = all_esdt.into_payments();
        if !all_esdt_payments.is_empty() {
            self.send().direct_multi(&caller, &all_esdt_payments);
        }
    }

    fn perform_single_call_from_arg(
        &self,
        arg: SingleCallArg<Self::Api>,
        total_egld: &mut BigUint,
        all_esdt: &mut UniquePayments<Self::Api>,
        opt_last_back_transfers: &mut Option<BackTransfers<Self::Api>>,
    ) {
        let (dest_address, payment_type, call_type, gas_limit, opt_exec_arg) = arg.into_tuple();
        require!(!dest_address.is_zero(), "May not send to zero address");

        // The only reason we keep gas limit separate is to save some encoding space. Don't need the whole u64 range for it.
        let opt_raw_call_args = opt_exec_arg.map(|exec_arg| RawCall {
            gas_limit,
            function_name: exec_arg.function_name,
            args: exec_arg.args,
        });

        match payment_type {
            PaymentType::None => {
                let opt_back_transfers =
                    self.perform_no_transfer(dest_address, call_type, opt_raw_call_args);
                *opt_last_back_transfers =
                    self.handle_back_transfers_if_any(total_egld, all_esdt, opt_back_transfers);
            }
            PaymentType::Egld { amount } => {
                require!(*total_egld >= amount, "Invalid EGLD amount");

                *total_egld -= &amount;

                let opt_back_transfers =
                    self.perform_egld_transfer(dest_address, call_type, amount, opt_raw_call_args);
                *opt_last_back_transfers =
                    self.handle_back_transfers_if_any(total_egld, all_esdt, opt_back_transfers);
            }
            PaymentType::FixedPayments { esdt_payments } => {
                for transfer in &esdt_payments {
                    require!(transfer.amount > 0, "Invalid ESDT value");

                    let deduct_result = all_esdt.deduct_payment(&transfer);
                    require!(deduct_result.is_ok(), "Invalid ESDT amount");
                }

                let opt_back_transfers = self.perform_multi_esdt_transfer(
                    dest_address,
                    call_type,
                    esdt_payments,
                    opt_raw_call_args,
                );
                *opt_last_back_transfers =
                    self.handle_back_transfers_if_any(total_egld, all_esdt, opt_back_transfers);
            }
            PaymentType::ReceivedPaymentsFromSc => {
                let last_back_transfers = match opt_last_back_transfers {
                    Some(back_transfers) => {
                        require!(
                            !back_transfers.esdt_payments.is_empty(),
                            "Only EGLD received or no ESDT received. May only use this feature with ESDT"
                        );

                        BackTransfers {
                            total_egld_amount: back_transfers.total_egld_amount.clone(),
                            esdt_payments: back_transfers.esdt_payments.clone(),
                        }
                    }
                    None => sc_panic!("No payments received from SC"),
                };

                for transfer in &last_back_transfers.esdt_payments {
                    let deduct_result = all_esdt.deduct_payment(&transfer);
                    require!(deduct_result.is_ok(), "May not use these back transfers");
                }

                let opt_back_transfers = self.perform_multi_esdt_transfer(
                    dest_address,
                    call_type,
                    last_back_transfers.esdt_payments,
                    opt_raw_call_args,
                );
                *opt_last_back_transfers =
                    self.handle_back_transfers_if_any(total_egld, all_esdt, opt_back_transfers);
            }
        }
    }

    #[must_use]
    fn handle_back_transfers_if_any(
        &self,
        total_egld: &mut BigUint,
        all_esdt: &mut UniquePayments<Self::Api>,
        opt_back_transfers: Option<BackTransfers<Self::Api>>,
    ) -> Option<BackTransfers<Self::Api>> {
        opt_back_transfers.map(|back_transfers| {
            let returned_transfers = BackTransfers {
                total_egld_amount: back_transfers.total_egld_amount.clone(),
                esdt_payments: back_transfers.esdt_payments.clone(),
            };

            self.add_payments_received_from_sc(total_egld, all_esdt, back_transfers);

            returned_transfers
        })
    }

    fn add_payments_received_from_sc(
        &self,
        total_egld: &mut BigUint,
        all_esdt: &mut UniquePayments<Self::Api>,
        back_transfers: BackTransfers<Self::Api>,
    ) {
        *total_egld += back_transfers.total_egld_amount;

        for single_esdt in back_transfers.esdt_payments {
            all_esdt.add_payment(single_esdt);
        }
    }
}
