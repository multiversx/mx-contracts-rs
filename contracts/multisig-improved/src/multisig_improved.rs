#![no_std]

use action_types::execute_action::MAX_BOARD_MEMBERS;
use common_types::user_role::UserRole;

pub mod action_types;
pub mod check_signature;
pub mod common_functions;
pub mod common_types;
pub mod external;
pub mod ms_endpoints;
pub mod state;

multiversx_sc::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[multiversx_sc::contract]
pub trait Multisig:
    state::StateModule
    + common_functions::CommonFunctionsModule
    + check_signature::CheckSignatureModule
    + ms_endpoints::propose::ProposeEndpointsModule
    + ms_endpoints::perform::PerformEndpointsModule
    + ms_endpoints::discard::DiscardEndpointsModule
    + ms_endpoints::sign::SignEndpointsModule
    + ms_endpoints::callbacks::CallbacksModule
    + action_types::external_module::ExternalModuleModule
    + action_types::execute_action::ExecuteActionModule
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

    fn add_multiple_board_members(&self, new_board_members: ManagedVec<ManagedAddress>) -> usize {
        let new_board_members_len = new_board_members.len();
        require!(
            new_board_members_len <= MAX_BOARD_MEMBERS,
            "board size cannot exceed limit"
        );

        let mapper = self.user_ids();
        for new_member in &new_board_members {
            let user_id = mapper.insert_new(&new_member);
            self.user_id_to_role(user_id).set(UserRole::BoardMember);
        }

        self.num_board_members().set(new_board_members_len);

        new_board_members_len
    }
}
