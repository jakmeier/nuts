use super::*;
use crate::nut::Nut;

pub(crate) struct LifecycleChange {
    activity: UncheckedActivityId,
    is_active: bool,
}

impl ActivityContainer {
    pub(crate) fn is_active(&self, id: UncheckedActivityId) -> bool {
        self.active[id.index]
    }
    fn set_active(&mut self, id: UncheckedActivityId, active: bool) {
        self.active[id.index] = active
    }
}

impl Nut {
    pub(crate) fn set_active(&self, id: UncheckedActivityId, is_active: bool) {
        let event = LifecycleChange {
            activity: id,
            is_active,
        };
        self.deferred_events.push(event.into());
        self.catch_up_deferred_to_quiescence();
    }
    /// only access after locking with executing flag
    pub(crate) fn unchecked_lifecycle_change(&self, lifecycle_change: LifecycleChange) {
        let before = self
            .activities
            .try_borrow()
            .expect("Bug: This should not be possible to trigger from outside the library.")
            .is_active(lifecycle_change.activity);
        if before != lifecycle_change.is_active {
            self.activities
                .try_borrow_mut()
                .expect("Bug: This should not be possible to trigger from outside the library.")
                .set_active(lifecycle_change.activity, lifecycle_change.is_active);
            if lifecycle_change.is_active {
                self.publish_local(lifecycle_change.activity, Topic::enter(), ());
            } else {
                self.publish_local(lifecycle_change.activity, Topic::leave(), ());
            }
        }
    }
}
