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
    pub potlock_id: PotlockId,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub owner: ManagedAddress<M>,
    pub status: Status,
}

impl<M: ManagedTypeApi> Project<M> {
    pub fn new(
        potlock_id: PotlockId,
        name: ManagedBuffer<M>,
        description: ManagedBuffer<M>,
        owner: ManagedAddress<M>,
    ) -> Self {
        Project {
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

    #[view(potDonations)]
    #[storage_mapper("potDonations")]
    fn pot_donations(&self, potlock_id: PotlockId) -> MapMapper<ManagedAddress, EsdtTokenPayment>;

    #[view(projectDonations)]
    #[storage_mapper("projectDonations")]
    fn project_donations(
        &self,
        project_id: ProjectId,
    ) -> MapMapper<ManagedAddress, EsdtTokenPayment>;
}
