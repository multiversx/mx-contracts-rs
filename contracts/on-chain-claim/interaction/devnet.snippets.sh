WALLET="${PWD}/wallet.pem"
PROJECT="${PWD}"
PROXY=https://devnet-gateway.multiversx.com
CHAINID=D

DEPLOY_GAS="25000000"
SFT_IDENTIFIER=0x585354525245504149522d653162363733 #XSTRREPAIR-e1b673

CONTRACT_ADDRESS="erd1qqqqqqqqqqqqqpgqkm3wla3wk0yqk7lk725wee8yh0e2zeru76ls3gr0nj"

deploy() {
    mxpy --verbose contract deploy \
          --bytecode="output/on-chain-claim.mxsc.json" \
          --arguments ${SFT_IDENTIFIER} \
          --pem=${WALLET} \
          --gas-limit=${DEPLOY_GAS} \
          --proxy=${PROXY} \
          --chain=${CHAINID} \
          --recall-nonce \
          --send \
          --outfile="deploy-devnet.interaction.json" || return

    TRANSACTION=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    mxpy --verbose contract upgrade ${CONTRACT_ADDRESS} \
          --bytecode="output/on-chain-claim.mxsc.json" \
          --pem=${WALLET} \
          --gas-limit=${DEPLOY_GAS} \
          --proxy=${PROXY} \
          --chain=${CHAINID} \
          --recall-nonce \
          --send \
          --outfile="upgrade-devnet.interaction.json" || return

    TRANSACTION=$(mxpy data parse --file="upgrade-devnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="upgrade-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${ADDRESS}
    mxpy data store --key=upgradeTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}
