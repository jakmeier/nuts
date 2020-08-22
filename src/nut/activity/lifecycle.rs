use super::*;
use crate::nut::Nut;

// @ START-DOC ACTIVITY_LIFECYCLE
/// An activity starts in an active lifecycle status.
/// The only other status is inactive in the current version of Nuts.
///
/// The inactive status can be used to put activities to sleep temporarily.
/// While inactive, the activity will not be notified of events it has subscribed to.
/// A subscription filter can been used to change this behavior.
/// (See [`subscribe_masked`](struct.ActivityId.html#method.subscribe_masked))
///
/// If the status of a changes from active to inactive, the corresponding [`on_leave`](struct.ActivityId.html#method.on_leave) and [`on_leave_domained`](struct.ActivityId.html#method.on_leave_domained) subscriptions will be called.
///
/// If the status of a changes from inactive to active, the corresponding [`on_enter`](struct.ActivityId.html#method.on_enter) and [`on_enter_domained`](struct.ActivityId.html#method.on_enter_domained) subscriptions will be called.0
///
// @ END-DOC ACTIVITY_LIFECYCLE
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum LifecycleStatus {
    /// The normal status. Every Activity starts with this status.
    Active,
    /// Inactive / Sleeping
    Inactive,
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
            .expect("Bug: This should not be possible to trigger from outside the library.")
            .status(lifecycle_change.activity);
        if before != lifecycle_change.status {
            self.activities
                .try_borrow_mut()
                .expect("Bug: This should not be possible to trigger from outside the library.")
                .set_status(lifecycle_change.activity, lifecycle_change.status);
            if !before.is_active() && lifecycle_change.status.is_active() {
                self.publish_local(lifecycle_change.activity, Topic::enter(), ());
            } else if before.is_active() && !lifecycle_change.status.is_active() {
                self.publish_local(lifecycle_change.activity, Topic::leave(), ());
            }
        }
    }
}
