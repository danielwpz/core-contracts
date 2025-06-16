# Voting Contract

The purpose of this contract is solely for validators to vote on whether to unlock
token transfer. Validators can call `vote` to vote for yes with the amount of stake they wish
to put on the vote.

**Voting Deadline:**
A voting deadline is set during the contract's initialization.
- **Before the deadline**: Voting is permitted.
- **After the deadline**: Voting is no longer allowed. Users can only query the voting results.
- **Automatic Failure**: If the required voting threshold (more than 2/3 of the stake) is not met by the deadline, the voting outcome is automatically set to 'failed'.

If there are more than 2/3 of the stake at any given moment voting for yes, the voting is done.
After the voting is finished, or the deadline is passed, no one can further modify the contract.

## Implementation Details

The deadline functionality is implemented using the `deadline` field in the contract's state.
- The `deadline` is set during the `instantiate` function call.
- The `try_vote` function checks the current block time against the `deadline` to ensure voting is only allowed before the deadline.
- The `query_result` function can be called at any time to get the current voting status.
- Unit tests for deadline-related scenarios are included in `src/lib.rs`.
