use crate::nut::Nut;
use crate::*;
use core::any::Any;

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
    pub(super) fn local<MSG: Any>(msg: MSG, id: UncheckedActivityId, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Local(id),
            msg: Box::new(msg),
            topic,
        }
    }
}

impl Nut {
    /// only access after locking with executing flag
    pub(crate) fn unchecked_broadcast(&self, broadcast: BroadcastInfo) {
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
