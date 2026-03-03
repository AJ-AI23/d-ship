//! Shared types and logic for d-ship contracts.
//!
//! Entity types mirror `schemas/` definitions (nShift-aligned). Use this crate to:
//! - Process and validate input via shared entity structs
//! - Create on-chain entities with ownership
//! - Charge callers or defined payment targets
//! - Apply granular access control
//! - Split data across storage units
#![no_std]

pub mod access;
pub mod agreement;
pub mod billing;
pub mod crypto;
pub mod entities;
pub mod ownership;
pub mod payment;
pub mod storage;
pub mod tracking_events;
pub mod validation;
