TransferRole, composability, fair launch

Problem statement: new economics needs special new models and rules for token and several actions one token and a user can take. Especially at early stages, let’s call fair launch, the tokens might have no fees for buying, and super high fees for selling, early LPs can be only burnt and not retrieved by the user. These are some of the rules for memecoins at the current stage, other ideas might arise as well.

With ESDT standard complex economics can be developed using the transferRole, as you can create tokens which are forced to go through a special contract in order for the transfer/any execution to go through.

Resolve: create a new transferSC1.0 standard and a template, which can be used by any project who wants special roles for its tokens. We can see a complicated version of this on the xMEX and the proxy-dex contract for xMEX. Using that knowledge, we can create a templated version for the transferSC1.0.

1. Transferring tokens from user to user: forwardTransfer@destination@extraArgs
The complete txData: MutliESDTNFTTransfer@transferContract@List<tokenID@nonce@value>@ forwardTransfer@destination@extraArguments

forwardTransfer@destination@extraArguments: will do extra checks (these extra checks regarding compliance or users or whatever is the job of the developer) on the received tokens and make a transferAndExecute to the destination, with the remaining tokens (like if a fee is taken out from one of those) and the received extra arguments. If the destination is a SC, the first argument out of those extras will be used as the function.

1.b. Extra Fees Model: We need to create a module for fees in which a map of actions and how much that action will cost in percentage. Some addresses/actions could have 0% fees as well.

2. Tokens and exchanges:
The transferSC1.0 has to be able to interact with the exchanges and all the dApps in a relatively easy way. As the token has a transferRole, every transaction in regards to an exchange will fail, thus we need a dynamic execution from the transferContract to handle exchange interactions.
If the user wants to enterLP, enterFarm or simply swap to and from the token it has to be done through the transferContract. Thus we define/implement a module through which multiple actions can be performed. This is true for swapping from both to and from the token.

Thus we make new endpoints:
forwardExecuteOnDest@destination@extraArguments: after verifying the multitransfer, compliance and taking fee from the special tokenID, this will call ExecuteOnDestContext - the results on the received payments will be sent to the users after an additional set of customized operations by the contract. In case of error the tx will fail and the user will get back his tokens.
All the received payments can be gathered through the new backTransferAPI - the transferSC contract could apply some fees on the received tokens as well, and the rest to send back to the user.

forwardAsyncCall@destination@extraArguments: does the same as above but with an asynchronous call, and will send back to the user on the callback. On callback we need to send back to the user all his tokens, even those which were taken out for a fee.


For a first usecase we could have the following example:
EnterLP can have 0 fees.
ExitLP - 50% of the LP is burnt, and only with the rest of 50% we call exitLP, this means more liquidity will remain in the liquidity pool.
Buying the new token - 0 fees.
Selling the new token - 10% fee - which will remain in this transferSC1.0.

3. Fair launches
For the initial launch of the tokens, we may create a new endpoint. When fairlaunch is active, the forwardExecuteOnDest and forwardAsyncCall can be stopped.
At one point we have in the xExchange liquidity pool contracts a module for protected launches. In which we set a limit of how much one user can buy. Also we had a decreasing FEE model on swapping for the first 24 hours. This could have a better place in this contract. 
Features: 
1. limit of how much one account can buy.
2. Limit how much one transaction can buy 
3. Add a decreasing fee model for buying
4. Add a decreasing fee model for selling

When there is a fair launch, in that period LP tokens will be completely burnt if someone calls exitLP. The liquidity remains in the Liquidity pool contract, and the LP token is burnt for the user who sent it.

You can read more here: https://agora.multiversx.com/t/transferrole-and-composability-transfercontract-mip-4/171
