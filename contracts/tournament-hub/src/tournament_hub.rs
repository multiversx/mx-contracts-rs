#![no_std]

multiversx_sc::imports!();

mod helpers;
mod models;
mod storage;
mod views;
mod tournament_logic {
    pub mod game_registration;
    pub mod results_management;
    pub mod spectator_betting;
    pub mod tournament_management;
}

// use tournament_logic::game_registration::GameRegistrationModule;
// use tournament_logic::results_management::ResultsManagementModule;
// use tournament_logic::spectator_betting::SpectatorBettingModule;
// use tournament_logic::tournament_management::TournamentManagementModule;
// use tournament_logic::views::ViewsModule;

#[multiversx_sc::contract]
pub trait TournamentHub: storage::StorageModule + helpers::HelperModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
