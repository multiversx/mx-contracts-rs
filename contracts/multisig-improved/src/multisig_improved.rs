#![no_std]

pub mod action_types;
pub mod check_signature;
pub mod common_types;
pub mod external;
pub mod state;

multiversx_sc::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[multiversx_sc::contract]
pub trait Multisig:
    state::StateModule
    + check_signature::CheckSignatureModule
    + action_types::propose::ProposeModule
    + action_types::sign::SignModule
    + action_types::perform::PerformModule
    + action_types::discard::DiscardActionModule
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
}
