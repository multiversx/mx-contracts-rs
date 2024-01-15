use crate::common::Percentage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct InitialLaunchBlocks {
    pub start: u64,
    pub end: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct InitialLaunchInfo<M: ManagedTypeApi> {
    pub account_buy_limit: BigUint<M>,
    pub tx_buy_limit: BigUint<M>,
    pub buy_fee_percentage_start: Percentage,
    pub buy_fee_percentage_end: Percentage,
    pub sell_fee_percentage_start: Percentage,
    pub sell_fee_percentage_end: Percentage,
}

#[multiversx_sc::module]
pub trait InitialLaunchModule {
    #[payable("*")]
    #[endpoint(buyToken)]
    fn buy_token(&self) {
        // TODO
    }

    #[payable("*")]
    #[endpoint(sellToken)]
    fn sell_token(&self) {
        // TODO
    }

    fn get_fee_percentage(
        &self,
        fee_percentage_start: Percentage,
        fee_percentage_end: Percentage,
    ) -> Percentage {
        let initial_launch_blocks = self.initial_launch_blocks().get();
        let current_block = self.blockchain().get_block_nonce();
        require!(
            current_block <= initial_launch_blocks.end,
            "Invalid buy/sell block"
        );

        let blocks_passed_in_penalty_phase = current_block - initial_launch_blocks.start;
        let blocks_diff = initial_launch_blocks.end - initial_launch_blocks.start;
        let percentage_diff = fee_percentage_end - fee_percentage_start;

        let penalty_percentage_increase =
            percentage_diff as u64 * blocks_passed_in_penalty_phase / (blocks_diff - 1);

        fee_percentage_start + penalty_percentage_increase as u32
    }

    fn require_not_initial_launch(&self) {
        let current_block = self.blockchain().get_block_nonce();
        let initial_launch_blocks = self.initial_launch_blocks().get();
        require!(
            current_block > initial_launch_blocks.end,
            "Cannot call this endpoint during initial launch"
        );
    }

    #[storage_mapper("initialLaunchBlocks")]
    fn initial_launch_blocks(&self) -> SingleValueMapper<InitialLaunchBlocks>;

    #[storage_mapper("initialLaunchInfo")]
    fn initial_launch_info(&self) -> SingleValueMapper<InitialLaunchInfo<Self::Api>>;

    #[storage_mapper("totalBought")]
    fn total_bought(&self, user_addr: ManagedAddress) -> SingleValueMapper<BigUint>;
}
