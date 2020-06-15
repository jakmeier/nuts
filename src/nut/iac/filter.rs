use crate::*;

/// Defines under which circumstances a subscribing activity should be called.
#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    pub active_only: bool,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self { active_only: true }
    }
}

impl ActivityContainer {
    /// Returns true if the call should go through (false if it should be filtered out)
    pub(crate) fn filter<A: Activity>(
        &self,
        id: ActivityId<A>,
        filter: &SubscriptionFilter,
    ) -> bool {
        !filter.active_only || self.is_active(id)
    }
}
