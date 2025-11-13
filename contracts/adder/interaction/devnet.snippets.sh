ALICE="${USERS}/alice.pem"
ADDRESS=$(mxpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-api.multiversx.com
PROJECT="../output/adder.wasm"

deploy() {
    mxpy --verbose contract deploy --bytecode=${PROJECT} --pem=${ALICE} --arguments 0 --send --outfile="deploy-devnet.interaction.json" --proxy=${PROXY} || return

    TRANSACTION=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

add() {
    read -p "Enter number: " NUMBER
    mxpy --verbose contract call ${ADDRESS} --pem=${ALICE} --function="add" --arguments ${NUMBER} --proxy=${PROXY} --send
}

getSum() {
    mxpy --verbose contract query ${ADDRESS} --function="getSum" --proxy=${PROXY}
}
