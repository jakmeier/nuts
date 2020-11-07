use crate::debug::DebugTypeName;
use crate::nut::Nut;
use crate::*;
use core::any::Any;

pub(crate) struct BroadcastInfo {
    address: BroadcastAddress,
    msg: Box<dyn Any>,
    topic: Topic,
    #[allow(dead_code)]
    type_name: DebugTypeName,
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
            type_name: DebugTypeName::new::<MSG>(),
        }
    }
    pub(super) fn local<MSG: Any>(msg: MSG, id: UncheckedActivityId, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Local(id),
            msg: Box::new(msg),
            topic,
            type_name: DebugTypeName::new::<MSG>(),
        }
    }
}

impl Nut {
    /// only access after locking with executing flag
    pub(crate) fn unchecked_broadcast(&self, broadcast: BroadcastInfo) {
        let mut managed_state = self.managed_state.borrow_mut();
        managed_state.set_broadcast(broadcast.msg);
        if let Some(handlers) = self.subscriptions.get().get(&broadcast.topic) {
            match broadcast.address {
                BroadcastAddress::Global => {
                    for sub in handlers.iter() {
                        #[cfg(debug_assertions)]
                        self.active_activity_name.set(Some(sub.type_name));
                        let f = &sub.handler;
                        f(&mut self.activities.borrow_mut(), &mut managed_state);
                    }
                    self.active_activity_name.set(None);
                }
                BroadcastAddress::Local(id) => {
                    for sub in handlers.iter_for(id) {
                        #[cfg(debug_assertions)]
                        self.active_activity_name.set(Some(sub.type_name));
                        let f = &sub.handler;
                        f(&mut self.activities.borrow_mut(), &mut managed_state);
                    }
                    #[cfg(debug_assertions)]
                    self.active_activity_name.set(None);
                }
            }
        }
        managed_state.clear_broadcast();
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for BroadcastInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.address {
            BroadcastAddress::Global => write!(f, "message of type {:?}", self.type_name),
            BroadcastAddress::Local(_) => write!(
                f,
                "message of type {:?} for single activity",
                self.type_name
            ),
        }
    }
}
