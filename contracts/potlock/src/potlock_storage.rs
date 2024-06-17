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

#[derive(
    TypeAbi, NestedEncode, NestedDecode, PartialEq, Debug, TopEncodeOrDefault, TopDecodeOrDefault,
)]
pub struct Potlock<M: ManagedTypeApi> {
    pub potlock_id: PotlocklId,
    pub token_identifier: TokenIdentifier<M>,
    pub fee: BigUint<M>,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub status: PotlockStatus,
    // pub payment: EsdtTokenPayment<M>,
}

impl<M: ManagedTypeApi> Default for Potlock<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: ManagedTypeApi> Potlock<M> {
    pub fn new(name: ManagedBuffer, description: ManagedBuffer) -> Self {
        Potlock{
            potlock_id: self.potlocks().len() + 1,
            token_identifier: TokenIdentifier::from(ManagedBuffer::default()).
            fee: BigUint::default(),
            name,
            description,
        }
    }
}

pub struct Project<M: ManagedTypeApi> {
    pub project_id: PotlocklId,
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub address: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> Default for Project<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: ManagedTypeApi> Project<M> {
    pub fn new(name: ManagedBuffer, description: ManagedBuffer) -> Self {
        Project{
            project_id: self.proposals().len() + 1,
            name,
            description,
        }
    }
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
    fn potlocks(&self) -> VecMapper<Potlock<Self::Api>>;

    #[view(getProjects)]
    #[storage_mapper("projects")]
    fn projects(&self) -> VecMapper<Project<Self::Api>>;

    #[view(feePotPayments)]
    #[storage_mapper("fee_pot_payments")]
    fn fee_pot_payments(&self, potlock_id: PotlockId, address: &ManagedAddress) -> UnorderedSetMapper<EsdtTokenPayment>;

    #[view(feeProjectPayments)]
    #[storage_mapper("fee_project_payments")]
    fn fee_project_payments(&self, project_id: ProjectId, address: &ManagedAddress) -> UnorderedSetMapper<EsdtTokenPayment>;
}
