use crate::models::{Tournament, TournamentStatus};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
// Specify the supertrait with its full path as required by the linter
pub trait TournamentManagementModule: crate::storage::StorageModule {
    #[endpoint(createTournament)]
    fn create_tournament(
        &self,
        tournament_id: ManagedBuffer,
        game_id: ManagedBuffer,
        entry_fee: BigUint,
        join_deadline: u64,
        play_deadline: u64,
    ) {
        require!(
            !self.active_tournaments().contains_key(&tournament_id),
            "Tournament ID already exists"
        );
        require!(
            self.registered_games().contains_key(&game_id),
            "Game not registered"
        );
        require!(
            join_deadline > self.blockchain().get_block_timestamp(),
            "Join deadline must be in the future"
        );
        require!(
            play_deadline > join_deadline,
            "Play deadline must be after join deadline"
        );

        let tournament = Tournament {
            game_id,
            status: TournamentStatus::Joining,
            entry_fee,
            participants: ManagedVec::new(),
            prize_pool: BigUint::zero(),
            join_deadline,
            play_deadline,
            final_podium: ManagedVec::new(),
            creator: self.blockchain().get_caller(),
        };

        self.active_tournaments().insert(tournament_id, tournament);
    }

    #[endpoint(joinTournament)]
    #[payable("EGLD")]
    fn join_tournament(&self, tournament_id: ManagedBuffer) {
        let payment = self.call_value().egld().clone_value();
        let caller = self.blockchain().get_caller();
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            self.active_tournaments().contains_key(&tournament_id),
            "Tournament does not exist"
        );

        let mut tournament = self.active_tournaments().get(&tournament_id).unwrap();
        let game_config = self.registered_games().get(&tournament.game_id).unwrap();

        // Check payment amount
        require!(payment == tournament.entry_fee, "Incorrect entry fee");

        // Check if player can join based on status and timing
        match tournament.status {
            TournamentStatus::Joining => {
                require!(
                    current_time <= tournament.join_deadline,
                    "Join deadline has passed"
                );
            }
            TournamentStatus::Playing => {
                require!(
                    game_config.allow_late_join,
                    "Late joining not allowed for this game"
                );
                require!(
                    current_time <= tournament.play_deadline,
                    "Play deadline has passed"
                );
            }
            _ => {
                sc_panic!("Cannot join tournament in current status");
            }
        }

        // Check if player is already participating
        for participant in tournament.participants.iter() {
            require!(participant.clone_value() != caller, "Player already joined");
        }

        // Add player and update prize pool
        tournament.participants.push(caller);
        tournament.prize_pool += &payment;

        self.active_tournaments().insert(tournament_id, tournament);
    }

    #[endpoint(startTournament)]
    fn start_tournament(&self, tournament_id: ManagedBuffer) {
        require!(
            self.active_tournaments().contains_key(&tournament_id),
            "Tournament does not exist"
        );

        let mut tournament = self.active_tournaments().get(&tournament_id).unwrap();

        require!(
            tournament.status == TournamentStatus::Joining,
            "Tournament is not in joining phase"
        );

        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time >= tournament.join_deadline,
            "Join deadline has not passed yet"
        );

        tournament.status = TournamentStatus::Playing;
        self.active_tournaments().insert(tournament_id, tournament);
    }
}
