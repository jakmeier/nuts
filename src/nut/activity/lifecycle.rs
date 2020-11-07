use super::*;
use crate::nut::{Nut, IMPOSSIBLE_ERR_MSG};

// @ START-DOC ACTIVITY_LIFECYCLE
/// Each activity has a lifecycle status that can be changed using [`set_status`](struct.ActivityId.html#method.set_status).
/// It starts with `LifecycleStatus::Active`.
/// In the current version of Nuts, the only other status is `LifecycleStatus::Inactive`.
///
/// The inactive status can be used to put activities to sleep temporarily.
/// While inactive, the activity will not be notified of events it has subscribed to.
/// A subscription filter can been used to change this behavior.
/// (See [`subscribe_masked`](struct.ActivityId.html#method.subscribe_masked))
///
/// If the status of a changes from active to inactive, the activity's [`on_leave`](struct.ActivityId.html#method.on_leave) and [`on_leave_domained`](struct.ActivityId.html#method.on_leave_domained) subscriptions will be called.
///
/// If the status of a changes from inactive to active, the activity's [`on_enter`](struct.ActivityId.html#method.on_enter) and [`on_enter_domained`](struct.ActivityId.html#method.on_enter_domained) subscriptions will be called.
///
// @ END-DOC ACTIVITY_LIFECYCLE
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum LifecycleStatus {
    /// The normal status. Every Activity starts with this status.
    Active,
    /// Inactive / Sleeping
    Inactive,
    /// Mark for deletion, the activity will be removed and `on_delete` called on it.
    /// Setting to this state twice will cause panics.
    Deleted,
}

pub(crate) struct LifecycleChange {
    activity: UncheckedActivityId,
    status: LifecycleStatus,
}

impl LifecycleStatus {
    /// Returns true iff the status is one that is considered to be active. (Only `LifecycleStatus::Active` at the moment)
    ///
    /// Use this method instead of a `==` comparison with `LifecycleStatus::Active`
    /// to keep compatibility with future versions of Nuts that may add other status variants.
    pub fn is_active(&self) -> bool {
        match self {
            Self::Active => true,
            Self::Inactive => false,
            Self::Deleted => false,
        }
    }
}

impl Nut {
    pub(crate) fn set_status(&self, id: UncheckedActivityId, status: LifecycleStatus) {
        let event = LifecycleChange {
            activity: id,
            status,
        };
        self.deferred_events.push(event.into());
        self.catch_up_deferred_to_quiescence();
    }
    /// only access after locking with executing flag
    pub(crate) fn unchecked_lifecycle_change(&self, lifecycle_change: &LifecycleChange) {
        let before = self
            .activities
            .try_borrow()
            .expect(IMPOSSIBLE_ERR_MSG)
            .status(lifecycle_change.activity);
        if before != lifecycle_change.status {
            assert_ne!(
                before,
                LifecycleStatus::Deleted,
                "Attempted to set activity status after it has been deleted."
            );
            self.activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .set_status(lifecycle_change.activity, lifecycle_change.status);
            if !before.is_active() && lifecycle_change.status.is_active() {
                self.publish_local(lifecycle_change.activity, Topic::enter(), ());
            } else if before.is_active() && !lifecycle_change.status.is_active() {
                self.publish_local(lifecycle_change.activity, Topic::leave(), ());
            }
        }
        if lifecycle_change.status == LifecycleStatus::Deleted {
            // Delete must be deferred in case the on_leave is hanging.
            self.deferred_events
                .push(nut::exec::Deferred::RemoveActivity(
                    lifecycle_change.activity,
                ));
        }
    }
    pub(crate) fn delete_activity(&self, id: UncheckedActivityId) {
        self.activities
            .try_borrow_mut()
            .expect(IMPOSSIBLE_ERR_MSG)
            .delete(
                id,
                &mut self
                    .managed_state
                    .try_borrow_mut()
                    .expect(IMPOSSIBLE_ERR_MSG),
            );
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for LifecycleChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tranisition to state: {:?}", self.status)
    }
}
