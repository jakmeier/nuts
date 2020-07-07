use crate::nut::Nut;
use crate::*;
use core::any::Any;
use core::sync::atomic::Ordering;

pub(crate) struct BroadcastInfo {
    address: BroadcastAddress,
    msg: Box<dyn Any>,
    topic: Topic,
}

enum BroadcastAddress {
    Local(UncheckedActivityId),
    Global,
}

impl BroadcastInfo {
    pub(super) fn global<MSG: Any>(msg: MSG, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Global,
            msg: Box::new(msg),
            topic,
        }
    }
    pub(super) fn local<A: Activity, MSG: Any>(msg: MSG, id: ActivityId<A>, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Local(id.id),
            msg: Box::new(msg),
            topic,
        }
    }
}

impl Nut {
    /// Delivers all queue broadcasts and all newly added broadcasts during that time period.
    ///
    /// If this is called in at a point of quiescence (no messages in flight)
    /// it will return also in such a point. (Queued messages are not in flight.)
    ///
    /// No guarantee is given for calls while a broadcast is ongoing (messages are in flight).
    /// It is perfectly valid (and the intended behavior) to do nothing when called while a broadcasting already.
    pub(super) fn broadcast(&self) {
        // A Nut only allows single-threaded access, relaxed ordering is fine.
        if !self.broadcasting.swap(true, Ordering::Relaxed) {
            self.broadcast_loop_unchecked();
            self.broadcasting.store(false, Ordering::Relaxed);
        }
    }
    /// only access after locking with broadcasting flag
    fn broadcast_loop_unchecked(&self) {
        while let Some(broadcast) = self.deferred_broadcasts.pop() {
            let mut managed_state = self.managed_state.borrow_mut();
            managed_state.set_broadcast(broadcast.msg);
            if let Some(handlers) = self.subscriptions.borrow().get(&broadcast.topic) {
                match broadcast.address {
                    BroadcastAddress::Global => {
                        for f in handlers.iter() {
                            f(&mut self.activities.borrow_mut(), &mut managed_state);
                        }
                    }
                    BroadcastAddress::Local(id) => {
                        for f in handlers.iter_for(id) {
                            f(&mut self.activities.borrow_mut(), &mut managed_state);
                        }
                    }
                }
            }
            managed_state.clear_broadcast();
        }
    }
}
