WALLET_PEM="~/walletKey.pem"
PROXY="https://devnet-gateway.multiversx.com"
CHAIN_ID="D"

MB_WASM_PATH="~/mx-contracts-rs/contracts/mystery-box/output/mystery-box.wasm"

CONTRACT_ADDRESS="erd1qqqqqqqqqqqqqpgql3ustfa2ac3d47y496865xkfrcxy48465dsqfzmxx5"

MB_TOKEN=0x4d425346542d663538616430 #MBSFT-f58ad0
deployMysteryBoxSC() {
    mxpy --verbose contract deploy --recall-nonce \
        --bytecode=${MB_WASM_PATH} \
        --pem=${WALLET_PEM} \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --arguments ${MB_TOKEN} \
        --send || return
}


upgradeMysteryBoxSC() {
    mxpy --verbose contract upgrade ${CONTRACT_ADDRESS} --recall-nonce \
        --bytecode=${MB_WASM_PATH} \
        --pem=${WALLET_PEM} \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --arguments ${MB_TOKEN} \
        --send || return
}

## SetESDTRoles - must be called manually
## ESDT manager address: erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
## Transfer createRole
## eg. transferNFTCreateRole@4d425346542d663538616430@acdc50f2c02039ea224f85f67e06194492b3fdfd74b2909fde0dd26f57dea360@00000000000000000500fc7905a7aaee22daf8952e8faa1ac91e0c4a9ebaa360
## Set burn and addQuantityRoles
## eg. setSpecialRole@4d425346542d663538616430@00000000000000000500fc7905a7aaee22daf8952e8faa1ac91e0c4a9ebaa360@45534454526f6c654e4654437265617465@45534454526f6c654e46544275726e@45534454526f6c654e46544164645175616e74697479

###PARAMS
#1 Experience points
XP_TYPE=1
XP_REWARD_TOKEN=str:EGLD
XP_VALUE=75
XP_DESCRIPTION=str:ExperiencePoints
XP_CHANCE=4000
DEFAULT_GLOBAL_COOLDOWN=0x00
#2 Mystery box
MB_TYPE=2
MB_TOKEN=str:MBSFT-f58ad0
MB_VALUE=1
MB_DESCRIPTION=str:MysteryBox
MB_CHANCE=3999
#3 SFT
SFT_TYPE=3
SFT_TOKEN=str:MBSFT-f58ad0
SFT_VALUE=1
SFT_DESCRIPTION=str:SFT
SFT_CHANCE=500
#4 PercentValue
PERCENT_TYPE=4
PERCENT_TOKEN=str:EGLD
PERCENT_VALUE=300
PERCENT_DESCRIPTION=str:PercentReward
PERCENT_CHANCE=1000
#5 FixedValue
FV_TYPE=5
FV_TOKEN=str:EGLD
FV_VALUE=50000000000000000000
FV_DESCRIPTION=str:FixedValueReward
FV_CHANCE=1
FV_GLOBAL_COOLDOWN=1
#6 Custom reward
CUSTOM_TYPE=6
CUSTOM_TOKEN=str:EGLD
CUSTOM_VALUE=1
CUSTOM_DESCRIPTION=str:Combo1
CUSTOM_CHANCE=500
CUSTOM_GLOBAL_COOLDOWN=1
setupMysteryBox() {
    mxpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce \
        --pem=${WALLET_PEM} \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function="setupMysteryBox" \
        --arguments $XP_TYPE $XP_REWARD_TOKEN $XP_VALUE $XP_DESCRIPTION $XP_CHANCE $DEFAULT_GLOBAL_COOLDOWN \
        $MB_TYPE $MB_TOKEN $MB_VALUE $MB_DESCRIPTION $MB_CHANCE $DEFAULT_GLOBAL_COOLDOWN \
        $SFT_TYPE $SFT_TOKEN $SFT_VALUE $SFT_DESCRIPTION $SFT_CHANCE $DEFAULT_GLOBAL_COOLDOWN \
        $PERCENT_TYPE $PERCENT_TOKEN $PERCENT_VALUE $PERCENT_DESCRIPTION $PERCENT_CHANCE $DEFAULT_GLOBAL_COOLDOWN \
        $FV_TYPE $FV_TOKEN $FV_VALUE $FV_DESCRIPTION $FV_CHANCE $FV_GLOBAL_COOLDOWN \
        $CUSTOM_TYPE $CUSTOM_TOKEN $CUSTOM_VALUE $CUSTOM_DESCRIPTION $CUSTOM_CHANCE $CUSTOM_GLOBAL_COOLDOWN \
        --send || return
}

URI=0x68747470733a2f2f63646e2e6d756c746976657273782e636f6d2f78706f7274616c2f6d7973746572792d626f782f6d7973746572795f626f782e706e67
updateMysteryBoxUris() {
    mxpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce \
        --pem=${WALLET_PEM} \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function="updateMysteryBoxUris" \
        --arguments $URI \
        --send || return
}

###PARAMS
#1 - Mystery box amount
createMysteryBox() {
    mxpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce \
        --pem=${WALLET_PEM} \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function="createMysteryBox" \
        --arguments $1 \
        --send || return
}

###PARAMS
#1 - Mystery box token nonce
openMysteryBox() {
    user_address="$(mxpy wallet pem-address $WALLET_PEM)"
    method_name=str:openMysteryBox
    mxpy --verbose contract call $user_address --recall-nonce \
        --pem=${WALLET_PEM} \
        --gas-limit=10000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function="ESDTNFTTransfer" \
        --arguments $MB_TOKEN $1 $MB_VALUE $CONTRACT_ADDRESS $method_name \
        --send || return
}

getMysteryBoxTokenIdentifier() {
    mxpy --verbose contract query ${CONTRACT_ADDRESS} \
        --proxy=${PROXY} \
        --function="getMysteryBoxTokenIdentifier"
}

getWinningRates() {
    mxpy --verbose contract query ${CONTRACT_ADDRESS} \
        --proxy=${PROXY} \
        --function="getWinningRates"
}

getMysteryBoxUris() {
        mxpy --verbose contract query ${CONTRACT_ADDRESS} \
        --proxy=${PROXY} \
        --function="getMysteryBoxUris"
}
