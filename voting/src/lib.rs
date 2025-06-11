use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, AccountId, Balance};
use std::collections::HashMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct VotingContract {
    /// How much each validator votes
    votes: HashMap<AccountId, Balance>,
    /// Total voted balance so far.
    total_voted_stake: Balance,
    /// When the voting ended. `None` means the poll is still open.
    result: Option<U64>,
    /// Epoch height when the contract is touched last time.
    last_epoch_height: u64,
}

/// Voting contract for unlocking transfers. Once the majority of the stake holders agree to
/// unlock transfer, the time will be recorded and the voting ends.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct VotingContract {
    /// How much each validator votes
    votes: HashMap<AccountId, Balance>,
    /// Total voted balance so far.
    total_voted_stake: Balance,
    /// When the voting ended. `None` means the poll is still open.
    result: Option<WrappedTimestamp>,
    /// Epoch height when the contract is touched last time.
    last_epoch_height: EpochHeight,
}

impl Default for VotingContract {
    fn default() -> Self {
        Self {
            votes: HashMap::new(),
            total_voted_stake: 0,
            result: None,
            last_epoch_height: 0,
        }
    }
}

#[near_bindgen]
impl VotingContract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self::default()
    }

    /// Ping to update the votes according to current stake of validators.
    pub fn ping(&mut self) {
        assert!(self.result.is_none(), "Voting has already ended");
        let cur_epoch_height = env::epoch_height();
        if cur_epoch_height != self.last_epoch_height {
            self.votes.clear();
            self.total_voted_stake = 0;
            for account_id in env::validator_account_ids() {
                let account_current_stake = env::validator_stake(&account_id);
                if account_current_stake > 0 {
                    self.total_voted_stake += account_current_stake;
                    self.votes.insert(account_id, account_current_stake);
                }
            }
            self.check_result();
            self.last_epoch_height = cur_epoch_height;
        }
    }

    /// Check whether the voting has ended.
    fn check_result(&mut self) {
        assert!(
            self.result.is_none(),
            "check result is called after result is already set"
        );
        let total_stake = env::validator_total_stake();
        if self.total_voted_stake > 2 * total_stake / 3 {
            self.result = Some(U64(env::block_timestamp()));
        }
    }

    /// Method for validators to vote or withdraw the vote.
    /// Votes for if `is_vote` is true, or withdraws the vote if `is_vote` is false.
    pub fn vote(&mut self, is_vote: bool) {
        self.ping();
        if self.result.is_some() {
            return;
        }
        let account_id = env::predecessor_account_id();
        let account_stake = env::validator_stake(&account_id);
        if is_vote {
            assert!(account_stake > 0, "{} is not a validator", account_id);
            self.votes.insert(account_id, account_stake);
        } else {
            self.votes.remove(&account_id);
        }
        self.total_voted_stake = self.votes.values().sum();
        self.check_result();
    }

    /// Get the timestamp of when the voting finishes. `None` means the voting hasn't ended yet.
    pub fn get_result(&self) -> Option<U64> {
        self.result
    }

    /// Returns current a pair of `total_voted_stake` and the total stake.
    /// Note: as a view method, it doesn't recompute the active stake. May need to call `ping` to
    /// update the active stake.
    pub fn get_total_voted_stake(&self) -> (U128, U128) {
        (
            self.total_voted_stake.into(),
            env::validator_total_stake().into(),
        )
    }

    /// Returns all active votes.
    /// Note: as a view method, it doesn't recompute the active stake. May need to call `ping` to
    /// update the active stake.
    pub fn get_votes(&self) -> HashMap<AccountId, U128> {
        self.votes
            .iter()
            .map(|(account_id, stake)| (account_id.clone(), (*stake).into()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, get_context, set_env};
    use near_sdk::{testing_env, AccountId};
    use std::collections::HashMap;
    use std::iter::FromIterator;

    fn get_context_with_epoch_height(
        predecessor_account_id: AccountId,
        epoch_height: u64,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id("alice_near".parse().unwrap())
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id)
            .epoch_height(epoch_height);
        builder
    }

    #[test]
    #[should_panic(expected = "is not a validator")]
    fn test_nonvalidator_cannot_vote() {
        let context = get_context_with_epoch_height("bob.near".parse().unwrap(), 0);
        set_env(context);
        let mut contract = VotingContract::new();
        contract.vote(true);
    }

    #[test]
    #[should_panic(expected = "Voting has already ended")]
    fn test_vote_again_after_voting_ends() {
        let context = get_context_with_epoch_height("alice.near".parse().unwrap(), 0);
        set_env(context);
        let mut contract = VotingContract::new();
        contract.vote(true);
        assert!(contract.result.is_some());
        contract.vote(true);
    }

    #[test]
    fn test_voting_simple() {
        let validators = (0..10)
            .map(|i| (format!("test{}", i).parse().unwrap(), 10))
            .collect::<HashMap<_, _>>();
        let mut contract = VotingContract::new();

        for i in 0..7 {
            let context = get_context_with_epoch_height(format!("test{}", i).parse().unwrap(), 0);
            set_env(context);
            contract.vote(true);
            let mut context = get_context_with_epoch_height(format!("test{}", i).parse().unwrap(), 0);
            context.is_view = true;
            set_env(context);
            assert_eq!(
                contract.get_total_voted_stake(),
                (U128::from(10 * (i + 1)), U128::from(100))
            );
            assert_eq!(
                contract.get_votes(),
                (0..=i)
                    .map(|i| (format!("test{}", i).parse().unwrap(), U128::from(10)))
                    .collect::<HashMap<_, _>>()
            );
            assert_eq!(contract.votes.len() as u128, (i + 1) as u128);
            if i < 6 {
                assert!(contract.result.is_none());
            } else {
                assert!(contract.result.is_some());
            }
        }
    }

    #[test]
    fn test_voting_with_epoch_change() {
        let mut contract = VotingContract::new();

        for i in 0..7 {
            let context = get_context_with_epoch_height(format!("test{}", i).parse().unwrap(), i as u64);
            set_env(context);
            contract.vote(true);
            assert_eq!(contract.votes.len() as u64, i as u64 + 1);
            if i < 6 {
                assert!(contract.result.is_none());
            } else {
                assert!(contract.result.is_some());
            }
        }
    }

    #[test]
    fn test_validator_stake_change() {
        let mut validators = HashMap::from_iter(vec![
            ("test1".parse().unwrap(), 40),
            ("test2".parse().unwrap(), 10),
            ("test3".parse().unwrap(), 10),
        ]);
        let context = get_context_with_epoch_height("test1".parse().unwrap(), 1);
        set_env(context);

        let mut contract = VotingContract::new();
        contract.vote(true);
        validators.insert("test1".parse().unwrap(), 50);
        let context = get_context_with_epoch_height("test2".parse().unwrap(), 2);
        set_env(context);
        contract.ping();
        assert!(contract.result.is_some());
    }

    #[test]
    fn test_withdraw_votes() {
        let context = get_context_with_epoch_height("test1".parse().unwrap(), 1);
        set_env(context);
        let mut contract = VotingContract::new();
        contract.vote(true);
        assert_eq!(contract.votes.len(), 1);
        let context = get_context_with_epoch_height("test1".parse().unwrap(), 2);
        set_env(context);
        contract.vote(false);
        assert!(contract.votes.is_empty());
    }

    #[test]
    fn test_validator_kick_out() {
        let mut validators = HashMap::from_iter(vec![
            ("test1".parse().unwrap(), 40),
            ("test2".parse().unwrap(), 10),
            ("test3".parse().unwrap(), 10),
        ]);
        let context = get_context_with_epoch_height("test1".parse().unwrap(), 1);
        set_env(context);

        let mut contract = VotingContract::new();
        contract.vote(true);
        assert_eq!((contract.get_total_voted_stake().0).0, 40);
        validators.remove(&"test1".parse().unwrap());
        let context = get_context_with_epoch_height("test2".parse().unwrap(), 2);
        set_env(context);
        contract.ping();
        assert_eq!((contract.get_total_voted_stake().0).0, 0);
    }
}
