multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PotlockId = usize;
pub type ProjectId = usize;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq)]
pub enum PotlockStatus {
    None,
    Active,
    Inactive,
}

#[derive(TypeAbi, NestedEncode, NestedDecode, PartialEq, Debug, TopEncode, TopDecode)]
pub struct Pot<M: ManagedTypeApi> {
    pub potlock_id: PotlockId,
    pub token_identifier: TokenIdentifier<M>,
    pub fee: BigUint<M>,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    // pub status: PotlockStatus,
    // pub payment: EsdtTokenPayment<M>,
}

impl<M: ManagedTypeApi> Pot<M> {
    pub fn new(
        potlock_id: PotlockId,
        name: ManagedBuffer<M>,
        description: ManagedBuffer<M>,
    ) -> Self {
        Pot {
            potlock_id,
            token_identifier: TokenIdentifier::from(ManagedBuffer::default()),
            fee: BigUint::default(),
            name,
            description,
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
    #[view(getFeeTokenIdentifier)]
    #[storage_mapper("fee_token_identifier")]
    fn fee_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getFeeAmount)]
    #[storage_mapper("fee_amount")]
    fn fee_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getPotlocks)]
    #[storage_mapper("potlocks")]
    fn potlocks(&self) -> VecMapper<Pot<Self::Api>>;

    #[view(getProjects)]
    #[storage_mapper("projects")]
    fn projects(&self) -> VecMapper<Project<Self::Api>>;

    #[view(feePotPayments)]
    #[storage_mapper("fee_pot_proposer")]
    fn fee_pot_proposer(&self, potlock_id: PotlockId) -> SingleValueMapper<ManagedAddress>;

    #[view(feeAmountAcceptPots)]
    #[storage_mapper("fee_amount_accepted_pots")]
    fn fee_amount_accepted_pots(&self) -> SingleValueMapper<BigUint>;

    #[view(potDonations)]
    #[storage_mapper("pot_donations")]
    fn pot_donations(&self, project_id: ProjectId) -> MapMapper<ManagedAddress, EsdtTokenPayment>;

    #[view(projectDonations)]
    #[storage_mapper("project_donations")]
    fn project_donations(
        &self,
        project_id: ProjectId,
    ) -> MapMapper<ManagedAddress, EsdtTokenPayment>;
}
