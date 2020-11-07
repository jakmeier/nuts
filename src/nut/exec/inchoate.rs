//! When executing a broadcast, `activities` and `managed_state` is not available.
//! To still be able to add new activities and subscriptions during that time, temporary
//! structures are used to buffer additions. Theses are then merged in a deferred event.

use crate::{Activity, ActivityContainer, ActivityId, DomainId, LifecycleStatus};

#[derive(Default)]
pub(crate) struct InchoateActivityContainer {
    activities: ActivityContainer,
    offset: usize,
}

impl InchoateActivityContainer {
    pub(crate) fn inc_offset(&mut self) {
        debug_assert_eq!(self.activities.len(), 0);
        self.offset += 1;
    }
    pub(crate) fn flush(&mut self, final_activities: &mut ActivityContainer) {
        final_activities.append(&mut self.activities)
    }
}

// Delegation impl
impl InchoateActivityContainer {
    pub(crate) fn add<A: Activity>(
        &mut self,
        a: A,
        domain: DomainId,
        status: LifecycleStatus,
    ) -> ActivityId<A> {
        let mut aid = self.activities.add(a, domain, status);
        aid.id.index += self.offset;
        aid
    }
}
