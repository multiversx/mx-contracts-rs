use crate::{Nonce, Percentage, MAX_PERCENTAGE};

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum FeeType<M: ManagedTypeApi> {
    FixedAmount(BigUint<M>),
    Percentage(Percentage),
}

#[multiversx_sc::module]
pub trait FeeModule {
    #[only_owner]
    #[endpoint(setFeeForFractionalisingNft)]
    fn set_fee_for_fractionalizing_nft(
        &self,
        token_id: TokenIdentifier,
        nonce: Nonce,
        fee: BigUint,
    ) {
        self.fee_nft(&token_id, nonce).set(fee);
    }

    #[only_owner]
    #[endpoint(setFeeForFactionalisingCollection)]
    fn set_fee_for_fractionalizing_collection(&self, token_id: TokenIdentifier, fee: BigUint) {
        self.fee_collection(&token_id).set(fee);
    }

    #[only_owner]
    #[endpoint(setFeeForDepositBaskedOfGoods)]
    fn set_fee_for_deposit_basket_of_goods(&self, fee_percentage: Percentage) {
        require!(fee_percentage <= MAX_PERCENTAGE, "Invalid fee percentage");

        self.fee_basket().set(fee_percentage);
    }

    #[view(getFee)]
    fn get_fee(&self, token_id: &TokenIdentifier, nonce: Nonce) -> FeeType<Self::Api> {
        let fee_for_token = self.fee_nft(token_id, nonce).get();
        if fee_for_token > 0 {
            return FeeType::FixedAmount(fee_for_token);
        }

        let fee_collection = self.fee_collection(token_id).get();
        if fee_collection > 0 {
            return FeeType::FixedAmount(fee_collection);
        }

        let fee_percentage = self.fee_basket().get();
        FeeType::Percentage(fee_percentage)
    }

    #[storage_mapper("feeNft")]
    fn fee_nft(&self, token_id: &TokenIdentifier, nonce: Nonce) -> SingleValueMapper<BigUint>;

    #[storage_mapper("feeColl")]
    fn fee_collection(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getFeePercentageForBasketDeposit)]
    #[storage_mapper("feeBasket")]
    fn fee_basket(&self) -> SingleValueMapper<Percentage>;
}
