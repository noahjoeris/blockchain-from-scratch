//! The Dictator consensus engine considers any header valid as long as it is signed by the dictator.
//! The "signature" is made by simply attaching one of the Consensus Authorities from the root of this module.
//!
//! Notice that there is no iterating or "searching" for a correct seal. This save a lot of computation and
//! a lot of energy.
//!
//! Throughout this chapter we will avoid performing _Actual_ cryptographic calculations because they
//! require a crypto library which and overcoming its own learning curve, plus they distract from the
//! underlying consensus-related logic. Instead, we just use the `ConsensusAuthority` enum from the module root.

use super::{Consensus, ConsensusAuthority, Header};
/// Dictator consensus is an identity-based consensus algorithm. It specifies a single dictator
/// identity who is the only identity authorized to sign valid blocks. Any block signed by the
/// dictator is valid (at the consensus level), and any block not signed by the dictator is invalid.
struct DictatorConsensus {
    dictator: ConsensusAuthority,
}

impl Consensus for DictatorConsensus {
    type Digest = ConsensusAuthority;

    /// Check that the header is signed by the dictator
    fn validate(&self, _: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        return header.consensus_digest == self.dictator;
    }

    /// Sign the given partial header by the dictator
    fn seal(&self, _: &Self::Digest, partial_header: Header<()>) -> Option<Header<Self::Digest>> {
        let signed_header: Header<ConsensusAuthority> = Header {
            consensus_digest: self.dictator,
            height: partial_header.height,
            extrinsics_root: partial_header.extrinsics_root,
            state_root: partial_header.state_root,
            parent: partial_header.parent,
        };

        Some(signed_header)
    }
}
