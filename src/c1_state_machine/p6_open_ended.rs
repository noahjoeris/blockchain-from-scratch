//! Now is your chance to get creative. Choose a state machine that interests you and model it here.
//! Get as fancy as you like. The only constraint is that it should be simple enough that you can
//! realistically model it in an hour or two.
//!
//! Here are some ideas:
//! * Board games:
//!   * Chess
//!   * Checkers
//!   * Tic tac toe
//! * Beaurocracies:
//!   * Beauro of Motor Vehicles - maintains driving licenses and vehicle registrations.
//!   * Public Utility Provider - Customers open accounts, consume the utility, pay their bill periodically, maybe utility prices fluctuate
//!   * Land ownership registry
//! * Tokenomics:
//!   * Token Curated Registry
//!   * Prediction Market
//!   * There's a game where there's a prize to be split among players and the prize grows over time. Any player can stop it at any point and take most of the prize for themselves.
//! * Social Systems:
//!   * Social Graph
//!   * Web of Trust
//!   * Reputation System

use super::{StateMachine, User};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Proposal {
    id: u64,
    proposed_action: String,
    proposed_by: User,
    pending_until_time_unit: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum VoteType {
    Aye,
    Nay,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Vote {
    proposal_id: u64,
    vote: VoteType,
    user: User,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceState {
    proposals: Vec<Proposal>,
    votes: Vec<Vote>,
    time_units_passed: u64,
}

impl GovernanceState {
    fn new() -> GovernanceState {
        GovernanceState {
            proposals: vec![],
            votes: vec![],
            time_units_passed: 0,
        }
    }

    fn one_time_unit_passed(&mut self) {
        self.time_units_passed += 1;
    }

    fn vote_in_favor(&mut self, proposal_id: u64, user: User) {
        let vote = Vote {
            proposal_id,
            vote: VoteType::Aye,
            user,
        };
        self.votes.push(vote);
    }

    fn vote_against(&mut self, proposal_id: u64, user: User) {
        let vote = Vote {
            proposal_id,
            vote: VoteType::Nay,
            user,
        };
        self.votes.push(vote);
    }

    fn add_proposal(&mut self, proposed_action: String, user: User, pending_until_time_unit: u64) {
        let proposal = Proposal {
            id: self.proposals.len() as u64 + 1,
            proposed_action,
            pending_until_time_unit,
            proposed_by: user,
        };
        self.proposals.push(proposal);
    }

    fn proposal_exists_and_pending(&self, proposal_id: u64) -> bool {
        self.proposals
            .iter()
            .any(|p| p.id == proposal_id && p.pending_until_time_unit >= self.time_units_passed)
    }

    fn has_user_voted(&self, proposal_id: u64, user: &User) -> bool {
        self.votes
            .iter()
            .any(|v| v.proposal_id == proposal_id && &v.user == user)
    }
}

pub enum GovernanceAction {
    OneTimeUnitPassed,
    VoteInFavor(u64, User),         // proposal_id, user
    VoteAgainst(u64, User),         // proposal_id, user
    AddProposal(String, User, u64), // proposed_action, proposed_by, pending_until_time_unit
}

impl StateMachine for GovernanceState {
    type State = GovernanceState;
    type Transition = GovernanceAction;

    fn next_state(starting_state: &Self::State, t: &Self::Transition) -> Self::State {
        match t {
            GovernanceAction::OneTimeUnitPassed => {
                let mut new_state = starting_state.clone();
                new_state.one_time_unit_passed();
                new_state
            }

            GovernanceAction::VoteInFavor(proposal_id, user) => {
                if starting_state.proposal_exists_and_pending(*proposal_id)
                    && !starting_state.has_user_voted(*proposal_id, user)
                {
                    let mut new_state = starting_state.clone();
                    new_state.vote_in_favor(*proposal_id, user.clone());
                    new_state
                } else {
                    starting_state.clone()
                }
            }

            GovernanceAction::VoteAgainst(proposal_id, user) => {
                if starting_state.proposal_exists_and_pending(*proposal_id)
                    && !starting_state.has_user_voted(*proposal_id, user)
                {
                    let mut new_state = starting_state.clone();
                    new_state.vote_against(*proposal_id, user.clone());
                    new_state
                } else {
                    starting_state.clone()
                }
            }

            GovernanceAction::AddProposal(
                proposed_action,
                proposed_by,
                pending_until_time_unit,
            ) => {
                if *pending_until_time_unit >= starting_state.time_units_passed {
                    let mut new_state = starting_state.clone();
                    new_state.add_proposal(
                        proposed_action.clone(),
                        proposed_by.clone(),
                        *pending_until_time_unit,
                    );
                    new_state
                } else {
                    starting_state.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let state = GovernanceState::new();
        assert_eq!(state.proposals.len(), 0);
        assert_eq!(state.votes.len(), 0);
        assert_eq!(state.time_units_passed, 0);
    }

    #[test]
    fn test_add_proposal() {
        let state = GovernanceState::new();
        let new_state = GovernanceState::next_state(
            &state,
            &GovernanceAction::AddProposal(
                "Upgrade tokenomics to give Noah 90% of the transaction fees".to_string(),
                User::Noah,
                10,
            ),
        );
        assert_eq!(new_state.proposals.len(), 1);
        assert_eq!(new_state.proposals[0].proposed_by, User::Noah);
        assert_eq!(
            new_state.proposals[0].proposed_action,
            "Upgrade tokenomics to give Noah 90% of the transaction fees"
        );
        assert_eq!(new_state.proposals[0].pending_until_time_unit, 10);
    }

    #[test]
    fn test_vote_in_favor() {
        let state = GovernanceState::new();
        let state_with_proposal = GovernanceState::next_state(
            &state,
            &GovernanceAction::AddProposal(
                "Increase the block size limit from 1MB to 2MB to improve transaction throughput."
                    .to_string(),
                User::Alice,
                10,
            ),
        );
        let final_state = GovernanceState::next_state(
            &state_with_proposal,
            &GovernanceAction::VoteInFavor(1, User::Bob),
        );
        assert_eq!(final_state.votes.len(), 1);
        assert_eq!(final_state.votes[0].vote, VoteType::Aye);
        assert_eq!(final_state.votes[0].user, User::Bob);
    }

    #[test]
    fn test_invalid_id_voting() {
        let state = GovernanceState::new();
        let new_state =
            GovernanceState::next_state(&state, &GovernanceAction::VoteInFavor(1, User::Bob));
        assert_eq!(new_state.votes.len(), 0); // No proposals yet
    }

    #[test]
    fn test_duplicate_voting() {
        let state = GovernanceState::new();
        let state_with_proposal = GovernanceState::next_state(
            &state,
            &GovernanceAction::AddProposal(
                "Upgrade the smart contract protocol to support more complex dApps.".to_string(),
                User::Alice,
                10,
            ),
        );
        let state_after_first_vote = GovernanceState::next_state(
            &state_with_proposal,
            &GovernanceAction::VoteInFavor(1, User::Bob),
        );

        let first_duplicate = GovernanceState::next_state(
            &state_after_first_vote,
            &GovernanceAction::VoteInFavor(1, User::Bob),
        ); // Duplicate vote
        let second_duplicate = GovernanceState::next_state(
            &first_duplicate,
            &GovernanceAction::VoteAgainst(1, User::Bob),
        ); // Duplicate vote

        assert_eq!(second_duplicate.votes.len(), 1); // Should still be 1
    }

    #[test]
    fn test_time_advancement() {
        let state = GovernanceState::new();
        let state_with_proposal = GovernanceState::next_state(
            &state,
            &GovernanceAction::AddProposal(
                "Fund a development grant for improving network security and resilience."
                    .to_string(),
                User::Alice,
                5,
            ),
        );

        let mut final_state = state_with_proposal;
        for _ in 0..6 {
            final_state =
                GovernanceState::next_state(&final_state, &GovernanceAction::OneTimeUnitPassed);
            // Advance time
        }

        assert_eq!(final_state.time_units_passed, 6);
        assert_eq!(final_state.proposals.len(), 1);
    }

    #[test]
    fn test_voting_on_expired_proposal() {
        let state = GovernanceState::new();
        let proposal_lifetime = 5;

        // Add a proposal with a limited lifetime
        let state_with_proposal = GovernanceState::next_state(
            &state,
            &GovernanceAction::AddProposal(
                "I create a youtube video about OpenGov for 10k DOT".to_string(),
                User::Alice,
                proposal_lifetime,
            ),
        );

        // Advance time to just beyond the proposal's lifetime
        let mut state_after_expiration = state_with_proposal;
        for _ in 0..proposal_lifetime + 1 {
            state_after_expiration = GovernanceState::next_state(
                &state_after_expiration,
                &GovernanceAction::OneTimeUnitPassed,
            );
        }

        // Attempt to vote on the expired proposal
        let final_state = GovernanceState::next_state(
            &state_after_expiration,
            &GovernanceAction::VoteInFavor(1, User::Bob),
        );

        // Check if the state remains unchanged (vote should have no effect)
        assert_eq!(final_state.votes.len(), state_after_expiration.votes.len());
        assert!(!final_state.proposal_exists_and_pending(1));
    }
}
