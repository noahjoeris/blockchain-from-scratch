//! Proof of Work is very energy intensive but is decentralized. Dictator is energy cheap, but
//! is completely centralized. Let's achieve a middle ground by choosing a set of authorities
//! who can sign blocks as opposed to a single dictator. This arrangement is typically known as
//! Proof of Authority.
//!
//! In public blockchains, Proof of Authority is often moved even further toward the decentralized
//! and permissionless end of the spectrum by electing the authorities on-chain through an economic
//! game in which users stake tokens. In such a configuration it is often known as "Proof of Stake".
//! Even when using the Proof of Stake configuration, the underlying consensus logic is identical to
//! the proof of authority we are writing here.

use super::{Consensus, ConsensusAuthority, Header};

/// A Proof of Authority consensus engine. If any of the authorities have signed the block, it is valid.
pub struct SimplePoa {
    pub authorities: Vec<ConsensusAuthority>,
}

impl Consensus for SimplePoa {
    type Digest = ConsensusAuthority;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        return self.authorities.contains(&header.consensus_digest);
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        if self.authorities.is_empty() {
            return None;
        }

        let signed_header = Header {
            consensus_digest: self.authorities[0],
            height: partial_header.height,
            extrinsics_root: partial_header.extrinsics_root,
            state_root: partial_header.state_root,
            parent: partial_header.parent,
        };

        Some(signed_header)
    }
}

/// A Proof of Authority consensus engine. Only one authority is valid at each block height.
/// As ever, the genesis block does not require a seal. After that the authorities take turns
/// in order.
struct PoaRoundRobinByHeight {
    authorities: Vec<ConsensusAuthority>,
}

impl Consensus for PoaRoundRobinByHeight {
    type Digest = ConsensusAuthority;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        if header.height == 0 {
            return true;
        }

        let pos = (header.height - 1) as usize % self.authorities.len();
        return self.authorities[pos] == header.consensus_digest;
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        // Genesis block does not require a seal
        if partial_header.height == 0 {
            return None;
        }

        let pos = (partial_header.height - 1) as usize % self.authorities.len();
        let signed_header = Header {
            consensus_digest: self.authorities[pos],
            height: partial_header.height,
            extrinsics_root: partial_header.extrinsics_root,
            state_root: partial_header.state_root,
            parent: partial_header.parent,
        };

        Some(signed_header)
    }
}

/// Both of the previous PoA schemes have the weakness that a single dishonest authority can corrupt the chain.
/// * When allowing any authority to sign, the single corrupt authority can sign blocks with invalid transitions
///   with no way to throttle them.
/// * When using the round robin by height, their is throttling, but the dishonest authority can stop block production
///   entirely by refusing to ever sign a block at their height.
///
/// A common PoA scheme that works around these weaknesses is to divide time into slots, and then do a round robin
/// by slot instead of by height
struct PoaRoundRobinBySlot {
    authorities: Vec<ConsensusAuthority>,
}

/// A digest used for PoaRoundRobinBySlot. The digest contains the slot number as well as the signature.
/// In addition to checking that the right signer has signed for the slot, you must check that the slot is
/// always strictly increasing. But remember that slots may be skipped.
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
struct SlotDigest {
    slot: u64,
    signature: ConsensusAuthority,
}

impl Consensus for PoaRoundRobinBySlot {
    type Digest = SlotDigest;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        if header.height == 0 {
            return true;
        }

        if self.authorities.is_empty() {
            return false;
        }

        let pos = (header
            .consensus_digest
            .slot
            .checked_sub(1)
            .expect("slot need to be at least 1")) as usize
            % self.authorities.len();

        let expected_authority = self.authorities[pos];

        return expected_authority == header.consensus_digest.signature
            && header.consensus_digest.slot > parent_digest.slot;
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        // Genesis block does not require a seal and we need at least one authority
        if partial_header.height == 0 || self.authorities.is_empty() {
            return None;
        }

        let slot = parent_digest.slot + 1;
        let pos = (slot - 1) as usize % self.authorities.len();
        let signature = self.authorities[pos];

        let slot_digest = SlotDigest { slot, signature };

        let signed_header = Header {
            consensus_digest: slot_digest,
            height: partial_header.height,
            extrinsics_root: partial_header.extrinsics_root,
            state_root: partial_header.state_root,
            parent: partial_header.parent,
        };

        Some(signed_header)
    }
}

#[cfg(test)]

// Helper function to create a Header
fn create_header(digest: ConsensusAuthority, height: u64) -> Header<ConsensusAuthority> {
    Header {
        consensus_digest: digest,
        height,
        parent: 123,
        state_root: 123,
        extrinsics_root: 123,
    }
}

#[test]
fn simple_poa_validate() {
    let poa = SimplePoa {
        authorities: vec![ConsensusAuthority::Alice, ConsensusAuthority::Bob],
    };

    let valid_header = create_header(ConsensusAuthority::Alice, 1);
    let invalid_header = create_header(ConsensusAuthority::Charlie, 1);

    assert!(poa.validate(&ConsensusAuthority::Alice, &valid_header));
    assert!(!poa.validate(&ConsensusAuthority::Alice, &invalid_header));
}

#[test]
fn simple_poa_seal() {
    let poa = SimplePoa {
        authorities: vec![ConsensusAuthority::Alice],
    };

    let partial_header = Header::<()> {
        consensus_digest: (),
        height: 1,
        parent: 123,
        state_root: 123,
        extrinsics_root: 123,
    };

    if let Some(sealed_header) = poa.seal(&ConsensusAuthority::Alice, partial_header) {
        assert_eq!(sealed_header.consensus_digest, ConsensusAuthority::Alice);
    } else {
        panic!("Seal method failed");
    }
}

#[test]
fn poa_round_robin_validate() {
    let poa = PoaRoundRobinByHeight {
        authorities: vec![ConsensusAuthority::Alice, ConsensusAuthority::Bob],
    };

    // Test genesis block (height 0)
    let genesis_header = create_header(ConsensusAuthority::Alice, 0);
    assert!(
        poa.validate(&ConsensusAuthority::Alice, &genesis_header),
        "Genesis block should always be valid"
    );

    // Test validation for non-genesis blocks
    let valid_header_alice = create_header(ConsensusAuthority::Alice, 1);
    let valid_header_bob = create_header(ConsensusAuthority::Bob, 2);
    let invalid_header = create_header(ConsensusAuthority::Charlie, 3);

    assert!(
        poa.validate(&ConsensusAuthority::Alice, &valid_header_alice),
        "Header should be valid for Alice at height 1"
    );
    assert!(
        poa.validate(&ConsensusAuthority::Bob, &valid_header_bob),
        "Header should be valid for Bob at height 2"
    );
    assert!(
        !poa.validate(&ConsensusAuthority::Alice, &invalid_header),
        "Header should be invalid for Charlie at any height"
    );
}

#[test]
fn poa_round_robin_seal() {
    let poa = PoaRoundRobinByHeight {
        authorities: vec![ConsensusAuthority::Alice, ConsensusAuthority::Bob],
    };

    // Seal for non-genesis blocks
    let partial_header_1 = Header::<()> {
        height: 1,
        consensus_digest: (),
        parent: 123,
        state_root: 123,
        extrinsics_root: 123,
    };
    let partial_header_2 = Header::<()> {
        height: 2,
        extrinsics_root: 123,
        state_root: 123,
        parent: 123,
        consensus_digest: (),
    };
    let partial_header_3 = Header::<()> {
        height: 3,
        extrinsics_root: 123,
        state_root: 123,
        parent: 123,
        consensus_digest: (),
    };

    // Testing sealing for height 1 (Alice)
    if let Some(sealed_header_1) = poa.seal(&ConsensusAuthority::Alice, partial_header_1) {
        assert_eq!(
            sealed_header_1.consensus_digest,
            ConsensusAuthority::Alice,
            "Sealed header for height 1 should be Alice"
        );
    } else {
        panic!("Seal method failed for height 1");
    }

    // Testing sealing for height 2 (Bob)
    if let Some(sealed_header_2) = poa.seal(&ConsensusAuthority::Bob, partial_header_2) {
        assert_eq!(
            sealed_header_2.consensus_digest,
            ConsensusAuthority::Bob,
            "Sealed header for height 2 should be Bob"
        );
    } else {
        panic!("Seal method failed for height 2");
    }

    // Testing sealing for height 3 (Alice)
    if let Some(sealed_header_3) = poa.seal(&ConsensusAuthority::Alice, partial_header_3) {
        assert_eq!(
            sealed_header_3.consensus_digest,
            ConsensusAuthority::Alice,
            "Sealed header for height 3 should be Alice"
        );
    } else {
        panic!("Seal method failed for height 3");
    }

    // Test for genesis block (height 0)
    let genesis_partial_header = Header::<()> {
        height: 0,
        consensus_digest: (),
        parent: 123,
        state_root: 123,
        extrinsics_root: 123,
    };
    assert!(
        poa.seal(&ConsensusAuthority::Alice, genesis_partial_header)
            .is_none(),
        "Genesis block should not be sealed"
    );
}
