ALICE="${USERS}/alice.pem"
ADDRESS=$(mxpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-testnet)
PROXY=https://testnet-api.multiversx.com
PROJECT="../output/adder.wasm"

deploy() {
    mxpy --verbose contract deploy --bytecode=${PROJECT} --pem=${ALICE} --arguments 0 --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} || return

    TRANSACTION=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

add() {
    read -p "Enter number: " NUMBER
    mxpy contract call ${ADDRESS} --pem=${ALICE} --function="add" --arguments ${NUMBER} --send --proxy=${PROXY}
}

getSum() {
    mxpy contract query ${ADDRESS} --function="getSum" --proxy=${PROXY}
}
