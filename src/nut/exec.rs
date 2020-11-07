use crate::nut::activity::LifecycleChange;
use crate::nut::iac::publish::{BroadcastInfo, ResponseSlot};
use crate::nut::Nut;
use crate::DomainStoreData;
use crate::UncheckedActivityId;

pub(crate) mod fifo;
pub(crate) mod inchoate;

pub(crate) enum Deferred {
    Broadcast(BroadcastInfo),
    BroadcastAwaitingResponse(BroadcastInfo, ResponseSlot),
    Subscription(NewSubscription),
    OnDeleteSubscription(UncheckedActivityId, OnDelete),
    LifecycleChange(LifecycleChange),
    RemoveActivity(UncheckedActivityId),
    DomainStore(DomainStoreData),
    FlushInchoateActivities,
}
use core::sync::atomic::Ordering;

use super::{
    iac::subscription::{NewSubscription, OnDelete},
    IMPOSSIBLE_ERR_MSG,
};

impl Nut {
    /// Delivers all queue broadcasts (or other events) and all newly added broadcasts during that time period.
    ///
    /// If this is called in at a point of quiescence (no messages in flight)
    /// it will return also in such a point. (Queued messages are not in flight.)
    ///
    /// No guarantee is given for calls while a broadcast is ongoing (messages are in flight).
    /// It is perfectly valid (and the intended behavior) to do nothing when called while a executing already.
    pub(crate) fn catch_up_deferred_to_quiescence(&self) {
        // A Nut only allows single-threaded access, relaxed ordering is fine.
        if !self.executing.swap(true, Ordering::Relaxed) {
            self.unchecked_catch_up_deferred_to_quiescence();
            self.executing.store(false, Ordering::Relaxed);
        }
    }

    /// only access after locking with executing flag
    fn unchecked_catch_up_deferred_to_quiescence(&self) {
        while let Some(deferred) = self.deferred_events.pop() {
            #[cfg(debug_assertions)]
            let debug_message = format!("Executing: {:?}", deferred);
            debug_print!("{}", debug_message);

            #[cfg(not(debug_assertions))]
            self.exec_deferred(deferred);

            // Unfortunately, this currently does not seem to work on the web.
            // To have good web debugging, the nuts panic hook should be used.
            #[cfg(debug_assertions)]
            if let Err(panic_info) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    self.exec_deferred(deferred)
                }))
            {
                log_print!("Panic ocurred while nuts was executing. {}", debug_message);
                log_print!(
                    "Activity executing right now: {:?}",
                    self.active_activity_name.get()
                );
                std::panic::resume_unwind(panic_info);
            }
        }
    }
    fn exec_deferred(&self, deferred: Deferred) {
        match deferred {
            Deferred::Broadcast(b) => self.unchecked_broadcast(b),
            Deferred::BroadcastAwaitingResponse(b, slot) => {
                self.unchecked_broadcast(b);
                Nut::with_response_tracker_mut(|rt| rt.done(&slot));
            }
            Deferred::Subscription(sub) => {
                self.subscriptions.exec_new_subscription(sub);
            }
            Deferred::OnDeleteSubscription(id, sub) => {
                self.activities
                    .try_borrow_mut()
                    .expect(IMPOSSIBLE_ERR_MSG)
                    .add_on_delete(id, sub);
            }
            Deferred::LifecycleChange(lc) => self.unchecked_lifecycle_change(&lc),
            Deferred::RemoveActivity(id) => self.delete_activity(id),
            Deferred::DomainStore(d) => self.exec_domain_store(d),
            Deferred::FlushInchoateActivities => self
                .inchoate_activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .flush(&mut *self.activities.try_borrow_mut().expect(IMPOSSIBLE_ERR_MSG)),
        }
    }
}
impl Into<Deferred> for BroadcastInfo {
    fn into(self) -> Deferred {
        Deferred::Broadcast(self)
    }
}

impl Into<Deferred> for LifecycleChange {
    fn into(self) -> Deferred {
        Deferred::LifecycleChange(self)
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Deferred {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Broadcast(b) => write!(f, "Broadcasting {:?}", b),
            Self::BroadcastAwaitingResponse(b, _rs) => write!(f, "Broadcasting {:?}", b),
            Self::Subscription(sub) => write!(f, "{:?}", sub),
            Self::OnDeleteSubscription(_, _) => write!(f, "Adding new on delete listener"),
            Self::LifecycleChange(lc) => write!(f, "{:?}", lc),
            Self::RemoveActivity(_id) => write!(f, "Delete activity."),
            Self::DomainStore(ds) => write!(f, "{:?}", ds),
            Self::FlushInchoateActivities => write!(f, "Adding new activities previously deferred"),
        }
    }
}
