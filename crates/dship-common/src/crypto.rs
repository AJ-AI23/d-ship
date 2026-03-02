//! Signature verification helpers.
//!
//! In contracts, use `self.crypto().verify_ed25519(&pubkey, &message, &signature)` directly.
//! The message should be the output of `agreement::build_approval_message`, then hashed (keccak256
//! or sha256) before signing. Frontend and contract must use the same hash algorithm.
