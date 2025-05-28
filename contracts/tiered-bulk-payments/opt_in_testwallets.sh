#!/bin/bash

for i in `seq 1 40`;
do
    mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/alice.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
    mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/bob.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
    mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/carol.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
    mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/grace.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
    mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/frank.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
done
