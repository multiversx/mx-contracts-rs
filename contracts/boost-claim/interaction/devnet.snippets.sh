WALLET="${PWD}/wallet.pem"
PROJECT="${PWD}"
PROXY=https://devnet-gateway.multiversx.com
CHAINID=D

DEPLOY_GAS="25000000"
SFT_IDENTIFIER=0x585354525245504149522d653162363733 #XSTRREPAIR-e1b673

DIFFERENCE_BETWEEN_CLAIMS=0x2a30 # 3 hours -> 10800 seconds
PRIZE_1=0x5052495a4531 # PRIZE1
PRIZE_2=0x5052495a4532 # PRIZE2
PRIZE_3=0x5052495a4533 # PRIZE3
PRIZE_4=0x5052495a4534 # PRIZE4


deploy() {
    mxpy --verbose contract deploy \
          --bytecode="output/boost-claim.mxsc.json" \
          --pem=${WALLET} \
          --gas-limit=${DEPLOY_GAS} \
          --proxy=${PROXY} \
          --chain=${CHAINID} \
          --recall-nonce \
          --arguments ${DIFFERENCE_BETWEEN_CLAIMS} ${PRIZE_1} ${PRIZE_2} ${PRIZE_3} ${PRIZE_4} \
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
          --bytecode="output/boost-claim.mxsc.json" \
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
