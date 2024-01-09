//! In the previous chapter, we considered a hypothetical scenario where blocks must contain an even state root
//! in order to be valid. Now we will express that logic here as a higher-order consensus engine. It is higher-
//! order because it will wrap an inner consensus engine, such as PoW or PoA and work in either case.

use crate::hash;
use std::marker::PhantomData;

use super::{p1_pow::moderate_difficulty_pow, Consensus, Header};

/// A Consensus engine that requires the state root to be even for the header to be valid.
/// Wraps an inner consensus engine whose rules will also be enforced.
struct EvenOnly<Inner: Consensus> {
    /// The inner consensus engine that will be used in addition to the even-only requirement.
    inner: Inner,
}

impl<Inner: Consensus> Consensus for EvenOnly<Inner> {
    type Digest = Inner::Digest;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
        if !self.inner.validate(parent_digest, header) {
            return false;
        }

        return header.state_root % 2 == 0;
    }

    fn seal(
        &self,
        parent_digest: &Self::Digest,
        partial_header: Header<()>,
    ) -> Option<Header<Self::Digest>> {
        if partial_header.state_root % 2 != 0 {
            return None;
        }

        self.inner.seal(parent_digest, partial_header)
    }
}

/// Using the moderate difficulty PoW algorithm you created in section 1 of this chapter as the inner engine,
/// create a PoW chain that is valid according to the inner consensus engine, but is not valid according to
/// this engine because the state roots are not all even.
fn almost_valid_but_not_all_even() -> Vec<Header<u64>> {
    let pow = moderate_difficulty_pow();

    let mut headers = Vec::new();

    let first_partial_header: Header<()> = Header {
        parent: 123,
        height: 123,
        state_root: 123,
        extrinsics_root: 123,
        consensus_digest: (),
    };

    headers.push(pow.seal(&123, first_partial_header).unwrap());

    for i in 1..10 {
        let partial_header = Header {
            parent: hash(headers.last().unwrap()),
            height: headers.last().unwrap().height + 1,
            state_root: i,
            extrinsics_root: i,
            consensus_digest: (),
        };

        let header = pow
            .seal(&headers.last().unwrap().consensus_digest, partial_header)
            .unwrap();
        headers.push(header.clone());
    }

    headers
}

#[test]
fn test_almost_valid_but_not_all_even() {
    // Create an instance of EvenOnly with PoW as the inner consensus mechanism
    let pow = moderate_difficulty_pow();
    let even_only = EvenOnly { inner: pow };

    // Generate headers using the almost_valid_but_not_all_even function
    let headers = almost_valid_but_not_all_even();
    println!("headers: {:?}", headers);

    let mut parent_digest = 0u64;
    // Iterate over headers and validate each header using EvenOnly
    for header in headers {
        let is_valid_even_only = even_only.validate(&parent_digest, &header);
        let is_valid_pow = even_only.inner.validate(&parent_digest, &header);

        parent_digest = header.consensus_digest;

        // Assert that headers with an even state_root are valid, and odd ones are not
        assert_eq!(is_valid_even_only, header.state_root % 2 == 0);

        // Assert that headers are valid according to the inner consensus engine
        assert!(is_valid_pow);
    }
}
