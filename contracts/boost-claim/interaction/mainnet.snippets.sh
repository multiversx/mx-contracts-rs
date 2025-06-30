WALLET="${PWD}/wallet.pem"
PROJECT="${PWD}"
PROXY=https://gateway.multiversx.com
CHAINID=D

DEPLOY_GAS="20000000"

DIFFERENCE_BETWEEN_CLAIMS=0x3840 # 4 hours -> 14400 seconds
PRIZE_1=0x5052495a4531 # PRIZE1
PRIZE_2=0x5052495a4532 # PRIZE2
PRIZE_3=0x5052495a4533 # PRIZE3
PRIZE_4=0x5052495a4534 # PRIZE4

CONTRACT_ADDRESS=erd1qqqqqqqqqqqqqpgq0dsmyccxtlkrjvv0czyv2p4kcy72xvt3nzgq8j2q3y # Shard 0
# CONTRACT_ADDRESS=erd1qqqqqqqqqqqqqpgqeu0tkupaqr64h95jtnk7pnhjnjs4sn9l7y9smjl4n9 # Shard 1
# CONTRACT_ADDRESS=erd1qqqqqqqqqqqqqpgqmksd4gl3xau5eja42sp6qmrxewxgj0ny4d3qfksmrq # Shard 2

deploy() {
    mxpy --verbose contract deploy \
          --bytecode="output/boost-claim.wasm" \
          --pem=${WALLET} \
          --gas-limit=${DEPLOY_GAS} \
          --proxy=${PROXY} \
          --chain=${CHAINID} \
          --recall-nonce \
          --arguments ${DIFFERENCE_BETWEEN_CLAIMS} ${PRIZE_1} ${PRIZE_2} ${PRIZE_3} ${PRIZE_4} \
          --send \
          --outfile="deploy-mainnet.interaction.json" || return

    TRANSACTION=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-mainnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-mainnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    mxpy --verbose contract upgrade ${CONTRACT_ADDRESS} \
          --bytecode="output/boost-claim.wasm" \
          --pem=${WALLET} \
          --gas-limit=${DEPLOY_GAS} \
          --proxy=${PROXY} \
          --chain=${CHAINID} \
          --recall-nonce \
          --send \
          --outfile="upgrade-mainnet.interaction.json" || return

    TRANSACTION=$(mxpy data parse --file="upgrade-mainnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="upgrade-mainnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-mainnet --value=${ADDRESS}
    mxpy data store --key=upgradeTransaction-mainnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

addAdmin() {
     ADMIN=0x9ff3241fc3c4ffa009df088fdd3f33e4b10b24cfb9a28e525bc9c46e47b3e0e2

     mxpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce \
         --pem=${WALLET} \
         --gas-limit=10000000 \
         --proxy=${PROXY} --chain=${CHAIN_ID} \
         --function="addAdmin" \
         --arguments $ADMIN \
         --send || return
 }
