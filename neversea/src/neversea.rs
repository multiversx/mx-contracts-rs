#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::derive::contract]
pub trait Neversea {
    #[init]
    fn init(&self, initial_registration_fee: BigUint) {
        self.registration_fee().set(BigUint::from(initial_registration_fee));
    }

    /// Add desired amount to the storage variable.
    #[endpoint]
    fn register(&self) {
        let caller = self.blockchain().get_caller();
        let payment_amount = self.call_value().egld_value();
        require!(payment_amount == BigUint::from(self.registration_fee().get()), "Registration fee is incorrect; please check and try again");
        self.participants().insert(caller);
    }

    #[view(getParticipants)]
    #[storage_mapper("participants")]
    fn participants(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getRegistrationFee)]
    #[storage_mapper("registration_fee")]
    fn registration_fee(&self) -> SingleValueMapper<BigUint>;


}
