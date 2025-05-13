#!/bin/bash

mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/alice.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/bob.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/grace.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
mxpy contract call vibe1qqqqqqqqqqqqqpgqmgtyppxexk6y3wgs3h3xh52zflw2zq8ttuzquzaxnx --pem testwallets/grace.pem --recall-nonce --chain V --proxy https://proxy.vibechain.ai --gas-limit 499999999 --function optIn --send
