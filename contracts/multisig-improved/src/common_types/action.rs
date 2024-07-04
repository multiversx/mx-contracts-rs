use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type GasLimit = u64;
pub type Nonce = u64;

pub type ActionId = usize;
pub type GroupId = usize;

#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Eq, Clone, Copy, Debug,
)]
pub enum ActionStatus {
    Available,
    Aborted,
}

#[derive(NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct CallActionData<M: ManagedTypeApi> {
    pub to: ManagedAddress<M>,
    pub egld_amount: BigUint<M>,
    pub opt_gas_limit: Option<GasLimit>,
    pub endpoint_name: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct EsdtTransferExecuteData<M: ManagedTypeApi> {
    pub to: ManagedAddress<M>,
    pub tokens: PaymentsVec<M>,
    pub opt_gas_limit: Option<GasLimit>,
    pub endpoint_name: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct DeployArgs<M: ManagedTypeApi> {
    pub amount: BigUint<M>,
    pub source: ManagedAddress<M>,
    pub code_metadata: CodeMetadata,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Clone)]
pub enum Action<M: ManagedTypeApi> {
    Nothing,
    AddBoardMember(ManagedAddress<M>),
    AddProposer(ManagedAddress<M>),
    RemoveUser(ManagedAddress<M>),
    ChangeQuorum(usize),
    SendTransferExecuteEgld(CallActionData<M>),
    SendTransferExecuteEsdt(EsdtTransferExecuteData<M>),
    SendAsyncCall(CallActionData<M>),
    SCDeployFromSource(DeployArgs<M>),
    SCUpgradeFromSource {
        sc_address: ManagedAddress<M>,
        args: DeployArgs<M>,
    },
    AddModule(ManagedAddress<M>),
    RemoveModule(ManagedAddress<M>),
}

impl<M: ManagedTypeApi> Action<M> {
    /// Only pending actions are kept in storage,
    /// both executed and discarded actions are removed (converted to `Nothing`).
    /// So this is equivalent to `action != Action::Nothing`.
    pub fn is_pending(&self) -> bool {
        !matches!(*self, Action::Nothing)
    }

    pub fn is_nothing(&self) -> bool {
        matches!(*self, Action::Nothing)
    }

    pub fn is_async_call(&self) -> bool {
        matches!(*self, Action::SendAsyncCall(_))
    }

    pub fn is_sc_upgrade(&self) -> bool {
        matches!(
            self,
            Action::SCUpgradeFromSource {
                sc_address: _,
                args: _
            }
        )
    }
}

/// Not used internally, just to retrieve results via endpoint.
#[derive(TopEncode, TypeAbi)]
pub struct ActionFullInfo<M: ManagedTypeApi> {
    pub action_id: ActionId,
    pub group_id: GroupId,
    pub action_data: Action<M>,
    pub signers: ManagedVec<M, ManagedAddress<M>>,
}

#[cfg(test)]
mod test {
    use multiversx_sc_scenario::api::StaticApi;

    use super::Action;

    #[test]
    fn test_is_pending() {
        assert!(!Action::<StaticApi>::Nothing.is_pending());
        assert!(Action::<StaticApi>::ChangeQuorum(5).is_pending());
    }
}
