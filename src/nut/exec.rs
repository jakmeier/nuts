use crate::nut::activity::LifecycleChange;
use crate::nut::iac::publish::BroadcastInfo;
use crate::nut::Nut;

pub(crate) mod fifo;

pub(crate) enum Deferred {
    Broadcast(BroadcastInfo),
    LifecycleChange(LifecycleChange),
}
use core::sync::atomic::Ordering;

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
            match deferred {
                Deferred::Broadcast(b) => self.unchecked_broadcast(b),
                Deferred::LifecycleChange(lc) => self.unchecked_lifecycle_change(lc),
            }
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
