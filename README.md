# Core contracts

- [Lockup / Vesting contract](./lockup/)
- [Lockup Factory](./lockup-factory/)
- [Multisig contract](./multisig/)
- [Staking Pool / Delegation contract](./staking-pool/)
- [Staking Pool Factory](./staking-pool-factory/)
- [Voting Contract](./voting/)
- [Whitelist Contract](./whitelist/)

## Voting Contract

The **Voting Contract** is a specialized smart contract designed to enable validators to vote on critical network decisions, specifically for unlocking token transfers. This contract implements a stake-weighted voting mechanism that ensures network security and decentralized governance.

### Key Features

- **Validator-Only Voting**: Only active validators can participate in the voting process
- **Stake-Weighted Decisions**: Each validator's vote is weighted by their current stake amount
- **Supermajority Threshold**: Requires more than 2/3 of the total validator stake to pass a proposal
- **Immutable Results**: Once voting concludes, the contract becomes read-only and cannot be modified
- **Real-time Stake Updates**: Automatically adjusts vote weights based on current validator stakes across epochs

### How It Works

1. **Initialization**: The contract is deployed and initialized with an empty voting state
2. **Voting Process**: Validators call the `vote(true)` method to cast their vote in favor of unlocking transfers
3. **Stake Calculation**: The contract uses each validator's current stake as their voting power
4. **Threshold Check**: When total voted stake exceeds 2/3 of all validator stake, voting concludes
5. **Result Recording**: The timestamp of voting completion is permanently recorded
6. **Vote Withdrawal**: Validators can withdraw their vote by calling `vote(false)` before voting concludes

### Core Methods

- `vote(is_vote: bool)` - Cast or withdraw a vote (validator only)
- `ping()` - Update vote weights based on current validator stakes
- `get_result()` - Returns the timestamp when voting concluded (if finished)
- `get_total_voted_stake()` - Returns current voted stake vs total validator stake
- `get_votes()` - Returns all active votes with their stake amounts

### Use Cases

- **Network Upgrades**: Coordinating validator consensus for protocol changes
- **Transfer Unlocking**: Originally designed for enabling token transfers on the network
- **Governance Decisions**: Any critical decision requiring validator supermajority approval

The Voting Contract ensures that only decisions with broad validator support (representing significant network stake) can be implemented, maintaining the security and decentralized nature of the NEAR Protocol.

## Building and deploying

See [scripts](./scripts/) folder for details.

## Initializing Contracts with near-shell

When setting up the contract creating the contract account, deploying the binary, and initializing the state must all be done as an atomic step.  For example, in our tests for the lockup contract we initialize it like this:

```rust
pub fn init_lockup(
        &self,
        runtime: &mut RuntimeStandalone,
        args: &InitLockupArgs,
        amount: Balance,
    ) -> TxResult {
        let tx = self
            .new_tx(runtime, LOCKUP_ACCOUNT_ID.into())
            .create_account()
            .transfer(ntoy(35) + amount)
            .deploy_contract(LOCKUP_WASM_BYTES.to_vec())
            .function_call(
                "new".into(),
                serde_json::to_vec(args).unwrap(),
                200000000000000,
                0,
            )
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        outcome_into_result(res)
    }
```


To do this with near shell, first add a script like `deploy.js`:

```js
const fs = require('fs');
const account = await near.account("foundation");
const contractName = "lockup-owner-id";
const newArgs = {
  "lockup_duration": "31536000000000000",
  "lockup_start_information": {
    "TransfersDisabled": {
        "transfer_poll_account_id": "transfers-poll"
    }
  },
  "vesting_schedule": {
    "start_timestamp": "1535760000000000000",
    "cliff_timestamp": "1567296000000000000",
    "end_timestamp": "1661990400000000000"
  },
  "staking_pool_whitelist_account_id": "staking-pool-whitelist",
  "initial_owners_main_public_key": "KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7",
  "foundation_account_id": "near"
}
const result = account.signAndSendTransaction(
    contractName,
    [
        nearAPI.transactions.createAccount(),
        nearAPI.transactions.transfer("100000000000000000000000000"),
        nearAPI.transactions.deployContract(fs.readFileSync("res/lockup_contract.wasm")),
        nearAPI.transactions.functionCall("new", Buffer.from(JSON.stringify(newArgs)), 100000000000000, "0"),
    ]);
```

Then use the `near repl` command. Once at the command prompt, load the script:

```js
> .load deploy.js
```

Note: `nearAPI` and `near` are both preloaded to the repl's context.
