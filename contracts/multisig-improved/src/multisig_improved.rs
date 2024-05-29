#![no_std]

pub mod action_types;
pub mod common_types;
pub mod external;
pub mod state;

use state::{ActionId, ActionStatus};

multiversx_sc::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[multiversx_sc::contract]
pub trait Multisig:
    state::StateModule
    + action_types::propose::ProposeModule
    + action_types::sign::SignModule
    + action_types::perform::PerformModule
    + external::events::EventsModule
    + external::views::ViewsModule
    + multiversx_sc_modules::dns::DnsModule
{
    #[init]
    fn init(&self, quorum: usize, board: MultiValueEncoded<ManagedAddress>) {
        let board_vec = board.to_vec();
        let new_num_board_members = self.add_multiple_board_members(board_vec);

        let num_proposers = self.num_proposers().get();
        require!(
            new_num_board_members + num_proposers > 0,
            "board cannot be empty on init, no-one would be able to propose"
        );

        require!(
            quorum <= new_num_board_members,
            "quorum cannot exceed board size"
        );
        self.quorum().set(quorum);
    }

    #[upgrade]
    fn upgrade(&self) {}

    /// Allows the contract to receive funds even if it is marked as unpayable in the protocol.
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {}

    /// Clears storage pertaining to an action that is no longer supposed to be executed.
    /// Any signatures that the action received must first be removed, via `unsign`.
    /// Otherwise this endpoint would be prone to abuse.
    #[endpoint(discardAction)]
    fn discard_action_endpoint(&self, action_id: ActionId) {
        let (_, caller_role) = self.get_caller_id_and_role();
        require!(
            caller_role.can_discard_action(),
            "only board members and proposers can discard actions"
        );

        self.discard_action(action_id);
    }

    /// Discard all the actions with the given IDs
    #[endpoint(discardBatch)]
    fn discard_batch(&self, action_ids: MultiValueEncoded<ActionId>) {
        let (_, caller_role) = self.get_caller_id_and_role();
        require!(
            caller_role.can_discard_action(),
            "only board members and proposers can discard actions"
        );

        for action_id in action_ids {
            self.discard_action(action_id);
        }
    }

    fn discard_action(&self, action_id: ActionId) {
        require!(
            self.get_action_valid_signer_count(action_id) == 0,
            "cannot discard action with valid signatures"
        );
        self.abort_batch_of_action(action_id);
        self.clear_action(action_id);
    }

    fn abort_batch_of_action(&self, action_id: ActionId) {
        let batch_id = self.group_for_action(action_id).get();
        if batch_id != 0 {
            self.action_group_status(batch_id)
                .set(ActionStatus::Aborted);
        }
    }
}
