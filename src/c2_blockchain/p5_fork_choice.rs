//! Forks in the blockchain represent alternative histories of the system.
//! When forks arise in the blockchain, users need a way to decide which chain
//! they will consider best, for now. This is known as a "fork choice rule".
//! There are several meaningful notions of "best", so we introduce a trait
//! that allows multiple implementations.
//!
//! Since we have nothing to add to the Block or Header data structures in this lesson,
//! we will import them from the previous lesson.

use super::p4_batched_extrinsics::{Block, Header};
use crate::hash;

const THRESHOLD: u64 = u64::max_value() / 100;

/// Judge which blockchain is "best" when there are multiple candidates. There are several
/// meaningful notions of "best" which is why this is a trait instead of just a
/// method.
pub trait ForkChoice {
    /// Compare two chains, and return the "best" one.
    ///
    /// The chains are not assumed to start from the same genesis block, or even a
    /// genesis block at all. This makes it possible to compare entirely disjoint
    /// histories. It also makes it possible to compare _only_ the divergent part
    /// of sibling chains back to the last common ancestor.
    ///
    /// The chains are assumed to be valid, so it is up to the caller to check
    /// validity first if they are unsure.
    fn first_chain_is_better(chain_1: &[Header], chain_2: &[Header]) -> bool;

    /// Compare many chains and return the best one.
    ///
    /// It is always possible to compare several chains if you are able to compare
    /// two chains. Therefore this method has a provided implementation. However,
    /// it may be much more performant to write a fork-choice-specific implementation.
    fn best_chain<'a>(candidate_chains: &[&'a [Header]]) -> &'a [Header];
}

/// The "best" chain is simply the longest chain.
pub struct LongestChainRule;

impl ForkChoice for LongestChainRule {
    fn first_chain_is_better(chain_1: &[Header], chain_2: &[Header]) -> bool {
        chain_1.len() > chain_2.len()
    }

    fn best_chain<'a>(candidate_chains: &[&'a [Header]]) -> &'a [Header] {
        // Remember, this method is provided. You _can_ solve the exercise by
        // simply deleting this block. It is up to you to decide whether this fork
        // choice warrants a custom implementation.
        candidate_chains
            .iter()
            .max_by_key(|chain| chain.len())
            .unwrap()
    }
}

/// The best chain is the one with the most accumulated work.
///
/// In Proof of Work chains, each block contains a certain amount of "work".
/// Roughly speaking, the lower a block's hash is, the more work it contains,
/// because finding a block with a low hash requires, on average, trying more
/// nonces. Modeling the amount of work required to achieve a particular hash
/// is out of scope for this exercise, so we will use the not-really-right-but
/// conceptually-good-enough formula `work = THRESHOLD - block_hash`
pub struct HeaviestChainRule;

/// Mutates a block (and its embedded header) to contain more PoW difficulty.
/// This will be useful for exploring the heaviest chain rule. The expected
/// usage is that you create a block using the normal `Block.child()` method
/// and then pass the block to this helper for additional mining.
fn mine_extra_hard(block: &mut Block, threshold: u64) {
    let old_block_hash = hash(&block.header);

    for nonce in 0.. {
        block.header.consensus_digest = nonce;
        let new_block_hash = hash(&block.header);
        if old_block_hash > new_block_hash && new_block_hash < threshold {
            return;
        }
    }
}

impl ForkChoice for HeaviestChainRule {
    fn first_chain_is_better(chain_1: &[Header], chain_2: &[Header]) -> bool {
        let work_chain1: u64 = chain_1
            .iter()
            .map(|h| THRESHOLD.checked_sub(hash(h)).unwrap_or(0))
            .sum();
        let work_chain2: u64 = chain_2
            .iter()
            .map(|h| THRESHOLD.checked_sub(hash(h)).unwrap_or(0))
            .sum();

        print!("work_chain1: {}, work_chain2: {}", work_chain1, work_chain2);

        work_chain1 > work_chain2
    }

    fn best_chain<'a>(candidate_chains: &[&'a [Header]]) -> &'a [Header] {
        // Remember, this method is provided.
        candidate_chains
            .iter()
            .max_by_key(|chain| {
                chain
                    .iter()
                    .map(|h| THRESHOLD.checked_sub(hash(h)).unwrap_or(0))
                    .sum::<u64>()
            })
            .unwrap()
    }
}
/// The best chain is the one with the most blocks that have even hashes.
///
/// This exact rule is a bit contrived, but it does model a family of fork choice rules
/// that are useful in the real world. We just can't code them here because we haven't
/// implemented Proof of Authority yet. Consider the following real world examples
/// that have very similar implementations.
///
/// 1. Secondary authors. In each round there is one author who is supposed to author.
///    If that author fails to create a block, there is a secondary author who may do so.
///    The best chain is the one with the most primary-authored blocks.
///
/// 2. Interleaved Pow/PoA. In each round there is one author who is allowed to author.
///    Anyone else is allowed to mine a PoW-style block. The best chain is the one with
///    the most PoA blocks, and ties are broken by the most accumulated work.
pub struct MostBlocksWithEvenHash;

impl ForkChoice for MostBlocksWithEvenHash {
    fn first_chain_is_better(chain_1: &[Header], chain_2: &[Header]) -> bool {
        let even_hashes_chain1: u64 = chain_1.iter().filter(|h| hash(h) % 2 == 0).count() as u64;
        let even_hashes_chain2: u64 = chain_2.iter().filter(|h| hash(h) % 2 == 0).count() as u64;

        even_hashes_chain1 > even_hashes_chain2
    }

    fn best_chain<'a>(candidate_chains: &[&'a [Header]]) -> &'a [Header] {
        // Remember, this method is provided.
        candidate_chains
            .iter()
            .max_by_key(|chain| chain.iter().filter(|h| hash(h) % 2 == 0).count() as u64)
            .unwrap()
    }
}

// This lesson has omitted one popular fork choice rule:
// GHOST - Greedy Heaviest Observed SubTree
//
// I've omitted GHOST from here because it requires information about blocks that
// are _not_ in the chain to decide which chain is best. Therefore it does't work
// well with this relatively simple trait definition. We will return to the GHOST
// rule later when we have written a full blockchain client
//
// The GHOST rule was first published in 2013 by Yonatan Sompolinsky and Aviv Zohar.
// Learn more at https://eprint.iacr.org/2013/881.pdf

//

/// Build and return two different chains with a common prefix.
/// They should have the same genesis header. Both chains should be valid.
/// The first chain should be longer (have more blocks), but the second
/// chain should have more accumulated work.
///
/// Return your solutions as three vectors:
/// 1. The common prefix including genesis
/// 2. The suffix chain which is longer (non-overlapping with the common prefix)
/// 3. The suffix chain with more work (non-overlapping with the common prefix)
fn create_fork_one_side_longer_other_side_heavier() -> (Vec<Header>, Vec<Header>, Vec<Header>) {
    let mut prefix = vec![Header::genesis()];
    prefix.push(prefix.last().unwrap().child(hash(&[1, 2, 3]), 6));
    prefix.push(prefix.last().unwrap().child(hash(&[3, 4, 5]), 18));

    let mut suffix_1: Vec<Header> = vec![];

    for i in 1..=5 {
        let last_header = suffix_1.last().unwrap_or(&prefix.last().unwrap());
        suffix_1.push(last_header.child(hash(&[i]), 18 + i));
    }

    let mut suffix_2: Vec<Header> = vec![];

    for i in 1..=4 {
        let last_header = suffix_2.last().unwrap_or(&prefix.last().unwrap());
        let mut block_mined_extra_hard = Block {
            header: last_header.child(hash(&[i]), 28 + i),
            body: vec![i],
        };
        mine_extra_hard(&mut block_mined_extra_hard, THRESHOLD / 10); // 10 times harder
        suffix_2.push(block_mined_extra_hard.header);
    }

    (prefix, suffix_1, suffix_2)
}

#[test]
fn bc_5_longest_chain() {
    let g = Header::genesis();

    let h_a1 = g.child(hash(&[1]), 1);
    let h_a2 = h_a1.child(hash(&[2]), 2);
    let chain_1 = &[g.clone(), h_a1, h_a2];

    let h_b1 = g.child(hash(&[3]), 3);
    let chain_2 = &[g, h_b1];

    assert!(LongestChainRule::first_chain_is_better(chain_1, chain_2));

    assert_eq!(LongestChainRule::best_chain(&[chain_1, chain_2]), chain_1);
}

#[test]
fn bc_5_mine_to_custom_difficulty() {
    let g = Block::genesis();
    let mut b1 = g.child(vec![1, 2, 3]);

    // We want the custom threshold to be high enough that we don't take forever mining
    // but low enough that it is unlikely we accidentally meet it with the normal
    // block creation function
    let custom_threshold = u64::max_value() / 1000;
    mine_extra_hard(&mut b1, custom_threshold);

    assert!(hash(&b1.header) < custom_threshold);
}

#[test]
fn bc_5_heaviest_chain() {
    let g = Header::genesis();

    let mut i = 0;
    let h_a1 = loop {
        let header = g.child(hash(&[i]), i);
        // Extrinsics root hash must be higher than threshold (less work done)
        if hash(&header) > THRESHOLD {
            break header;
        }
        i += 1;
    };
    let chain_1 = &[g.clone(), h_a1];

    let h_b1 = loop {
        let header = g.child(hash(&[i]), i);
        // Extrinsics root hash must be lower than threshold (more work done)
        if hash(&header) < THRESHOLD {
            break header;
        }
        i += 1;
    };
    let chain_2 = &[g, h_b1];

    assert!(HeaviestChainRule::first_chain_is_better(chain_2, chain_1));

    assert_eq!(HeaviestChainRule::best_chain(&[chain_1, chain_2]), chain_2);
}

#[test]
fn bc_5_most_even_blocks() {
    let g = Header::genesis();

    let mut h_a1 = g.child(2, 0);
    for i in 0..u64::max_value() {
        h_a1 = g.child(2, i);
        if hash(&h_a1) % 2 == 0 {
            break;
        }
    }
    let mut h_a2 = g.child(2, 0);
    for i in 0..u64::max_value() {
        h_a2 = h_a1.child(2, i);
        if hash(&h_a2) % 2 == 0 {
            break;
        }
    }
    let chain_1 = &[g.clone(), h_a1, h_a2];

    let mut h_b1 = g.child(2, 0);
    for i in 0..u64::max_value() {
        h_b1 = g.child(2, i);
        if hash(&h_b1) % 2 != 0 {
            break;
        }
    }
    let mut h_b2 = g.child(2, 0);
    for i in 0..u64::max_value() {
        h_b2 = h_b1.child(2, i);
        if hash(&h_b2) % 2 != 0 {
            break;
        }
    }
    let chain_2 = &[g, h_b1, h_b2];

    assert!(MostBlocksWithEvenHash::first_chain_is_better(
        chain_1, chain_2
    ));

    assert_eq!(
        MostBlocksWithEvenHash::best_chain(&[chain_1, chain_2]),
        chain_1
    );
}

#[test]
fn bc_5_longest_vs_heaviest() {
    let (_, longest_chain, pow_chain) = create_fork_one_side_longer_other_side_heavier();

    assert!(LongestChainRule::first_chain_is_better(
        &longest_chain,
        &pow_chain
    ));

    assert_eq!(
        LongestChainRule::best_chain(&[&longest_chain, &pow_chain]),
        &longest_chain
    );

    let (_, longest_chain, pow_chain) = create_fork_one_side_longer_other_side_heavier();

    assert!(HeaviestChainRule::first_chain_is_better(
        &pow_chain,
        &longest_chain
    ));

    assert_eq!(
        HeaviestChainRule::best_chain(&[&longest_chain, &pow_chain]),
        &pow_chain
    );
}
