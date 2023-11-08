# Game lobby SC


## Overview
This smart contract covers the functionality of a basic game lobby under a set of rules imposed by the owner.

**A user** can create a new wager game with the following specifications:
- `waiting time`
- `minimum number of players`
- `maximum number of players`
- `wager`

and has to pay a fee for each new game. 

**The owner** can:
- `enable/disable` the contract for maintenance
- set the `game starting fee` amount
- set the `token id` for the currency of the SC (used for game starting fee, wager and reward)
- `send rewards` or return the wager to the users who participated in a specific game
- `set/remove admin` rights for a specific address 

Each player has to pay the `wager` amount set by the game creator in the `token id` set by the owner in order to join the game.

The game is considered `invalid` until the `minimum number of players` have joined the game and if the `waiting time` has passed, no more players can join.

The SC does not have any logic for calculating the winner, so it expects input from the owner with the winners' addresses and the percentage (*100) of the total reward (sum of wagers) won by each. 

**The game**:
- If the game is `invalid`, the `wager` amount will be returned to the players that have joined the game and the `game starting fee` will be returned to the creator
- If the game is `valid`, but no winners are provided, such in the case of a tie/draw, the contract will send back the `wager` amount paid by every player who joined
- If the game is `valid` and winners are provided, the SC will send the rewards to them, based on the input of the owner.

## Endpoints
### createGame
```rust
#[payable("*")]
#[endpoint(createGame)]
fn create_game(
    &self,
    waiting_time: u64,
    number_of_players_min: u64,
    number_of_players_max: u64,
    wager: BigUint,
    )
```
Creates a game with a new id using the parameters sent by the caller if the payment is right (payment should be equal to `game starting fee`). 
The SC calculates min and max from the parameters so you don't have to worry if you placed them wrong.


### joinGame
```rust
#[payable("*")]
#[endpoint(joinGame)]
fn join_game(&self, game_id: u64) 
```
Caller can join a game with an existing game id if the payment is right (payment should be equal to `wager`).


### sendReward
```rust
#[endpoint(sendReward)]
fn send_reward(
    &self,
    game_id: u64,
    winners: OptionalValue<MultiValueEncoded<(ManagedAddress, u64)>>,
    )
```
Owner or admins can send the rewards for the players through this endpoint.


**winners:**
- the address of the winner
- the percentage of the reward pool the winner is entitled to * 100, (e.g: for 12.53%, the owner should send 1253 as parameter)

### claimBackWager
```rust
#[endpoint(claimBackWager)]
fn claim_back_wager(&self, game_id: u64)
```
Caller can manually claim back the `wager` if the game is `invalid` and the `waiting time` has passed (in case the owner has not already sent the wager through the **sendReward** endpoint)