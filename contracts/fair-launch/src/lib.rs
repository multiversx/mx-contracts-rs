#![no_std]

use common::Percentage;
use initial_launch::InitialLaunchBlocks;

use crate::{common::MAX_FEE_PERCENTAGE, initial_launch::InitialLaunchInfo};

multiversx_sc::imports!();

pub mod common;
pub mod exchange_actions;
pub mod initial_launch;
pub mod token_info;
pub mod transfer;

#[multiversx_sc::contract]
pub trait FairLaunch:
    common::CommonModule
    + exchange_actions::ExchangeActionsModule
    + initial_launch::InitialLaunchModule
    + token_info::TokenInfoModule
    + transfer::TransferModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// Percentages have to be between 0 and 10_000
    /// Start percentage >= End percentage
    #[init]
    fn init(
        &self,
        initial_launch_duration_blocks: u64,
        account_buy_limit: BigUint,
        tx_buy_limit: BigUint,
        buy_fee_percentage_start: Percentage,
        buy_fee_percentage_end: Percentage,
        sell_fee_percentage_start: Percentage,
        sell_fee_percentage_end: Percentage,
    ) {
        require!(
            initial_launch_duration_blocks > 0,
            "Invalid launch duration blocks"
        );
        require!(tx_buy_limit <= account_buy_limit, "Invalid limits");
        require!(
            buy_fee_percentage_start >= buy_fee_percentage_end
                && buy_fee_percentage_start <= MAX_FEE_PERCENTAGE,
            "Invalid percentage"
        );
        require!(
            sell_fee_percentage_start >= sell_fee_percentage_end
                && sell_fee_percentage_start <= MAX_FEE_PERCENTAGE,
            "Invalid percentage"
        );

        let start_block = self.blockchain().get_block_nonce();
        let end_block = start_block + initial_launch_duration_blocks;
        self.initial_launch_blocks().set(InitialLaunchBlocks {
            start: start_block,
            end: end_block,
        });

        let launch_info = InitialLaunchInfo {
            account_buy_limit,
            tx_buy_limit,
            buy_fee_percentage_start,
            buy_fee_percentage_end,
            sell_fee_percentage_start,
            sell_fee_percentage_end,
        };
        self.initial_launch_info().set(launch_info);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
