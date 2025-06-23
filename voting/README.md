# Voting Contract

The purpose of this contract is solely for validators to vote on whether to unlock
token transfer. Validators can call `vote` to vote for yes with the amount of stake they wish
to put on the vote.

Voting Deadline:
A voting deadline is set upon contract initialization.
- Before the deadline, voting is allowed.
- After the deadline, voting is forbidden. Users can only query voting results.
- If insufficient votes are cast before the deadline, the voting outcome is considered a failure.

If there are more than 2/3 of the stake at any given moment voting for yes, the voting is done.
After the voting is finished, no one can further modify the contract.
