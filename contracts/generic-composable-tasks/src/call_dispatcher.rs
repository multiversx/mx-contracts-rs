use crate::{
    raw_call::common::{FunctionName, GasLimit, PaymentsVec, RawArgs},
    unique_payments::UniquePayments,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum PaymentType<M: ManagedTypeApi> {
    None,
    Egld { amount: BigUint<M> },
    FixedPayments { esdt_payments: PaymentsVec<M> },
    ReceivedPaymentsFromSc,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum CallType {
    SimpleTransfer,
    Sync,
    Async,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
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
    crate::raw_call::common::CommonModule + multiversx_sc_modules::pause::PauseModule
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

        let mut total_egld = self.call_value().egld().clone_value();
        let mut all_esdt =
            UniquePayments::new_from_payments(self.call_value().all_esdt_transfers().clone_value());

        for arg in args {
            let (dest_address, payment_type, call_type, gas_limit, opt_exec_arg) = arg.into_tuple();
            require!(!dest_address.is_zero(), "May not send to zero address");

            match payment_type {
                PaymentType::None => todo!(),
                PaymentType::Egld { amount } => todo!(),
                PaymentType::FixedPayments { esdt_payments } => todo!(),
                PaymentType::ReceivedPaymentsFromSc => todo!(),
            }
        }
    }

    fn perform_no_transfer(&self) {}

    fn perform_egld_transfer(&self) {}

    fn perform_multi_esdt_transfer(&self) {}

    fn perform_transfer_from_received_payments(&self) {}

    fn add_payments_received_from_sc(
        &self,
        total_egld: &mut BigUint,
        all_esdt: &mut UniquePayments<Self::Api>,
        new_egld: BigUint,
        new_esdt: PaymentsVec<Self::Api>,
    ) {
        *total_egld += new_egld;

        for single_esdt in new_esdt {
            all_esdt.add_payment(single_esdt);
        }
    }
}
