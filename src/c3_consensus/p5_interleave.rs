//! PoW and PoA each have their own set of strengths and weaknesses. Many chains are happy to choose
//! one of them. But other chains would like consensus properties that fall in between. To achieve this
//! we could consider interleaving PoW blocks with PoA blocks. Some very early designs of Ethereum considered
//! this approach as a way to transition away from PoW.

/// A Consensus engine that alternates back and forth between PoW and PoA sealed blocks.
///
/// Odd blocks are PoW
/// Even blocks are PoA
///
use super::{p1_pow::Pow, p3_poa::SimplePoa, Consensus, ConsensusAuthority, Header};
struct AlternatingPowPoa {
    pow: Pow,
    poa: SimplePoa,
}

/// In order to implement a consensus that can be sealed with either work or a signature,
/// we will need an enum that wraps the two individual digest types.
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum PowOrPoaDigest {
    Pow(u64),
    Poa(ConsensusAuthority),
}

impl AlternatingPowPoa {
    /// Create a new instance of the Alternating PoW/PoA consensus engine.
    pub fn new(pow: Pow, poa: SimplePoa) -> Self {
        AlternatingPowPoa { pow, poa }
    }
}

impl From<u64> for PowOrPoaDigest {
    fn from(nonce: u64) -> Self {
        PowOrPoaDigest::Pow(nonce)
    }
}

impl TryFrom<PowOrPoaDigest> for u64 {
    type Error = ();

    fn try_from(digest: PowOrPoaDigest) -> Result<Self, Self::Error> {
        match digest {
            PowOrPoaDigest::Pow(nonce) => Ok(nonce),
            _ => Err(()),
        }
    }
}

impl From<ConsensusAuthority> for PowOrPoaDigest {
    fn from(authority: ConsensusAuthority) -> Self {
        PowOrPoaDigest::Poa(authority)
    }
}

impl TryFrom<PowOrPoaDigest> for ConsensusAuthority {
    type Error = ();

    fn try_from(digest: PowOrPoaDigest) -> Result<Self, Self::Error> {
        match digest {
            PowOrPoaDigest::Poa(authority) => Ok(authority),
            _ => Err(()),
        }
    }
}

impl Consensus for AlternatingPowPoa {
    type Digest = PowOrPoaDigest;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        if header.height % 2 == 0 {
            // PoA
            let consensus_digest_result: Result<ConsensusAuthority, _> =
                header.consensus_digest.try_into();

            if consensus_digest_result.is_err() {
                return false;
            }

            let poa_header = Header {
                parent: header.parent,
                height: header.height,
                state_root: header.state_root,
                extrinsics_root: header.extrinsics_root,
                consensus_digest: consensus_digest_result.unwrap(),
            };

            self.poa.validate(&ConsensusAuthority::Alice, &poa_header) // parent digest is not used in SimplePoA
        } else {
            // PoW
            let consensus_digest_result: Result<u64, _> = header.consensus_digest.try_into();

            if consensus_digest_result.is_err() {
                return false;
            }

            let pow_header: Header<u64> = Header {
                parent: header.parent,
                height: header.height,
                state_root: header.state_root,
                extrinsics_root: header.extrinsics_root,
                consensus_digest: consensus_digest_result.unwrap(),
            };
            self.pow.validate(&0, &pow_header) // parent digest is not used in PoW
        }
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        if partial_header.height % 2 == 0 {
            // PoA

            let sealed_header = self
                .poa
                .seal(&ConsensusAuthority::Alice, partial_header)
                .unwrap();

            Some(Header {
                parent: sealed_header.parent,
                height: sealed_header.height,
                state_root: sealed_header.state_root,
                extrinsics_root: sealed_header.extrinsics_root,
                consensus_digest: PowOrPoaDigest::Poa(sealed_header.consensus_digest),
            })
        } else {
            // PoW
            let sealed_header = self.pow.seal(&0, partial_header).unwrap();

            Some(Header {
                parent: sealed_header.parent,
                height: sealed_header.height,
                state_root: sealed_header.state_root,
                extrinsics_root: sealed_header.extrinsics_root,
                consensus_digest: PowOrPoaDigest::Pow(sealed_header.consensus_digest),
            })
        }
    }
}
