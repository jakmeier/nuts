//! When executing a broadcast, `activities` and `managed_state` is not available.
//! To still be able to add new activities and subscriptions during that time, temporary
//! structures are used to buffer additions. Theses are then merged in a deferred event.

use crate::{
    nut::iac::managed_state::ManagedState, nut::iac::subscription::OnDelete, Activity,
    ActivityContainer, ActivityId, DomainId, LifecycleStatus, UncheckedActivityId,
};

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
    pub(crate) fn status(&self, mut id: UncheckedActivityId) -> LifecycleStatus {
        id.index -= self.offset;
        self.activities.status(id)
    }
    pub(crate) fn set_status(&mut self, mut id: UncheckedActivityId, status: LifecycleStatus) {
        id.index -= self.offset;
        self.activities.set_status(id, status)
    }
    pub(crate) fn add_on_delete(&mut self, mut id: UncheckedActivityId, f: OnDelete) {
        id.index -= self.offset;
        self.activities.add_on_delete(id, f)
    }
    pub(crate) fn delete(&mut self, mut id: UncheckedActivityId, managed_state: &mut ManagedState) {
        id.index -= self.offset;
        self.activities.delete(id, managed_state)
    }
    pub(crate) fn len(&self) -> usize {
        self.activities.len()
    }
}
