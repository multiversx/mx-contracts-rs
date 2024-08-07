use crate::potlock_admin_interactions::{ProjectPercentage, MAX_PERCENTAGE};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PotlockId = usize;
pub type ProjectId = usize;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Debug, NestedEncode, NestedDecode)]
pub enum Status {
    Inactive,
    Active,
}

#[derive(TypeAbi, NestedEncode, NestedDecode, PartialEq, Debug, TopEncode, TopDecode)]
pub struct Pot<M: ManagedTypeApi> {
    pub potlock_id: PotlockId,
    pub proposer: ManagedAddress<M>,
    pub token_identifier: TokenIdentifier<M>,
    pub fee: BigUint<M>,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub status: Status,
}

impl<M: ManagedTypeApi> Pot<M> {
    pub fn new(
        potlock_id: PotlockId,
        proposer: ManagedAddress<M>,
        name: ManagedBuffer<M>,
        description: ManagedBuffer<M>,
    ) -> Self {
        Pot {
            potlock_id,
            proposer,
            token_identifier: TokenIdentifier::from(ManagedBuffer::default()),
            fee: BigUint::default(),
            name,
            description,
            status: Status::Inactive,
        }
    }
}

#[derive(TypeAbi, NestedEncode, NestedDecode, PartialEq, Debug, TopEncode, TopDecode)]
pub struct Project<M: ManagedTypeApi> {
    pub project_id: ProjectId,
    pub potlock_id: PotlockId,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub owner: ManagedAddress<M>,
    pub status: Status,
}

impl<M: ManagedTypeApi> Project<M> {
    pub fn new(
        project_id: ProjectId,
        potlock_id: PotlockId,
        name: ManagedBuffer<M>,
        description: ManagedBuffer<M>,
        owner: ManagedAddress<M>,
    ) -> Self {
        Project {
            project_id,
            potlock_id,
            name,
            description,
            owner,
            status: Status::Inactive,
        }
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct UserDonations<M: ManagedTypeApi> {
    pub user: ManagedAddress<M>,
    pub donations: EsdtTokenPayment<M>,
}

#[multiversx_sc::module]
pub trait PotlockStorage {
    fn is_valid_potlock_id(&self, potlock_id: PotlockId) -> bool {
        potlock_id >= 1 && potlock_id <= self.potlocks().len()
    }

    fn require_potlock_exists(&self, potlock_id: PotlockId) {
        require!(
            self.is_valid_potlock_id(potlock_id) && !self.potlocks().item_is_empty(potlock_id),
            "Potlock doesn't exist!",
        )
    }

    fn require_potlock_is_active(&self, potlock_id: PotlockId) {
        let potlock = self.potlocks().get(potlock_id);
        require!(potlock.status == Status::Active, "Pot is not active!",)
    }

    fn is_valid_project_id(&self, project_id: ProjectId) -> bool {
        project_id >= 1 && project_id <= self.projects().len()
    }

    fn require_project_exists(&self, project_id: ProjectId) {
        require!(
            self.is_valid_project_id(project_id) && !self.projects().item_is_empty(project_id),
            "Project doesn't exist!",
        )
    }

    fn require_project_is_active(&self, project_id: ProjectId) {
        let project = self.projects().get(project_id);
        require!(project.status == Status::Active, "Project is not active!",)
    }

    fn require_correct_percentages(
        &self,
        project_percentages: MultiValueEncoded<ProjectPercentage>,
    ) {
        let mut total_perc: u64 = 0;
        for pp in project_percentages {
            let (_, perc) = pp.into_tuple();
            total_perc += perc;
        }
        require!(
            total_perc <= MAX_PERCENTAGE,
            "Total percentages more than 100%"
        );
    }

    #[view(getFeeTokenIdentifier)]
    #[storage_mapper("feeTokenIdentifier")]
    fn fee_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getFeeAmount)]
    #[storage_mapper("feeAmount")]
    fn fee_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getPotlocks)]
    #[storage_mapper("potlocks")]
    fn potlocks(&self) -> VecMapper<Pot<Self::Api>>;

    #[view(getProjects)]
    #[storage_mapper("projects")]
    fn projects(&self) -> VecMapper<Project<Self::Api>>;

    #[view(feeAmountAcceptPots)]
    #[storage_mapper("feeAmountAcceptedPots")]
    fn fee_amount_accepted_pots(&self) -> SingleValueMapper<BigUint>;

    #[view(potDonations)]
    #[storage_mapper("potDonations")]
    fn pot_donations(&self, project_id: ProjectId) -> MapMapper<ManagedAddress, EsdtTokenPayment>;

    #[view(projectDonations)]
    #[storage_mapper("projectDonations")]
    fn project_donations(
        &self,
        project_id: ProjectId,
    ) -> MapMapper<ManagedAddress, EsdtTokenPayment>;
}
