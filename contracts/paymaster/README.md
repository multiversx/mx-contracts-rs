# Paymaster SC

## Overview

**Definition of problem**: the user needs to have a gas token (eGLD in case of MultiversX) to be able to do anything on the blockchain.
This is a hard concept to explain to non-crypto users.
Also gives some headaches for crypto users as well for the first steps of onboarding.
Imagine those people who bridge their NFTs or those who bridge USDC to a new account.
The new account can’t do anything until it does not have some eGLD to pay for the transactions.

In order for users to onboard faster to xPortal we introduced the concept of relayed transactions (we have v1, v2 and soon v3 of it).
This means a relayer can wrap the user transaction inside a relayedTX and in this case the relayer pays for the gas, but the user’s transaction is getting executed.
Right now, the relayers we use are free of charge, thus they do this service for new users for free, but they actually need eGLD to do the transactions.
When we speak about a few hundred users, this is not an issue, but when you want to scale up relaying transactions to thousands/millions of users, it becomes unsustainable.

## Implementation

Paymaster is a SC that makes relayed transactions sustainable.
This means that a user who doesn't own EGLD (native token) can make transactions by paying a fee in another token.

The Paymaster's objective is twofold:
- take a fee and send it to the Relayers;
- execute what the user wants to be executed.


The contract has only one endpoint: `forward_execution` and can be called by anyone.
The user will use `MultiESDTNFTTransfer` support to send multiple payments:
- first payment is always the fee that will be sent to the Relayer;
- rest of the payments will be what users want to send.

One example of userTX is:
```
MultiESDTNFTTransfer@paymasterSCAddr@feeTokenID@nonce@value@listofOther(tokenID,nonce,value)@forwardExecution@relayerAddr@destination@endpoint@extraArguments

```

After sending the Relayer the fee, `forward_execution` endpoint will make an *asynchronous call* to the destination.
The destionation can be a user or a smart contract.

We register a callback to the *asynchronous call*. In case of failure the paymaster SC sends the tokens back to the user.