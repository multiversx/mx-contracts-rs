# Paymaster SC

## Overview

Paymaster is a SC that makes relayed transactions sustainable.
This means that a user who doesn't own EGLD (native token) can make transactions by paying a fee in another token.

The Paymaster's objective is twofold:
- take a fee and send it to the Relayers;
- execute what the user wants to be executed.

## Implementation

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