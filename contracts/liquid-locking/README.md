# Liquid Locking Contract

This contract facilitates staking and unstaking of ESDT tokens.

The owner of the contract will set an unbonding period which will represent the number of epochs after unstaking when the tokens are available for claim.

Stake, unstake and unbond work with calls that contain multiple tokens


## Endpoints

### init

```rust
    #[init]
    fn init(&self, unbond_period: u64);
```

The init function is called when deploying/upgrading the smart contract. It sets the unbonding period of the tokens locked in the contract.

### set_unbond_period

```rust
    #[only_owner]
    #[endpoint]
    fn set_unbond_period(&self, unbond_period: u64);
```

The ```set_unbond_period``` endpoint allows changing the unbond period by the owner of te contract.

### whitelist_token

```rust
    #[only_owner]
    #[endpoint]
    fn whitelist_token(&self, token: &TokenIdentifier);
```

This endpoint is also a only owner endpoint used to whitelist an __ESDT__ token in order to be lockable inside the contract.

### blacklist_token

```rust
    #[only_owner]
    #[endpoint]
    fn blacklist_token(&self, token: &TokenIdentifier);
```

Just like the previous endpoint the only owner endpoint ```blacklist_token``` can be used to remove an __ESDT__ token from the whitelist in order to be lockable inside the contract. The already locked tokens of the type of the one removed will remain in the contract and will be further available for unlocking and unbonding, but will not be further available for locking.

### lock

```rust
    #[payable("*")]
    #[endpoint]
    fn lock(&self);
```

```lock``` allows a user to stake whitelisted tokens inside the contract. The endpoint can be called with multiple whitelisted tokens as payment.

### unlock

```rust
    #[endpoint]
    fn unlock(&self, tokens: ManagedVec<EsdtTokenPayment<Self::Api>>) 
```

```unlock``` allows a user to unstake locked tokens from the contract. By calling this endpoint the requested tokens will enter a unbonding period only after which will be available for claiming. The endpoint requires a list of tokens as parameter, representing the tokens desired for unstaking. Each unstake instantiated a separate unbonding period for the desired amount of tokens.

### unbond
```rust
    #[endpoint]
    fn unbond(&self, tokens: ManagedVec<TokenIdentifier>);
```

```unlock``` allows a user to claim unstaken tokens that passed the unbonding period. In case of multiple instances of the same token only the ones that passed their respective unbonding period will be claimed. The endpoint requires a list of tokens as parameter, representing the tokens desired for unstaking.
