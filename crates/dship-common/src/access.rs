//! Granular access control. Roles and permissions per entity/action.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{ManagedAddress, ManagedBuffer},
};

/// Access rule: who can perform what action.
#[derive(Clone)]
pub struct AccessRule<M: ManagedTypeApi> {
    pub subject: ManagedAddress<M>,
    pub action: ManagedBuffer<M>, // e.g. "create", "update", "void"
    pub resource: ManagedBuffer<M>, // e.g. "shipment", "parcel"
}

/// Check if subject is allowed to perform action on resource.
/// Implementations can use owner check, role lists, or config-driven rules.
pub fn can_perform<M: ManagedTypeApi>(
    subject: &ManagedAddress<M>,
    action: &ManagedBuffer<M>,
    resource: &ManagedBuffer<M>,
    owner: &ManagedAddress<M>,
) -> bool
{
    // Owner can do anything on own resources
    if subject == owner {
        return true;
    }
    // Extend: role-based, config-driven allow/deny lists
    false
}
