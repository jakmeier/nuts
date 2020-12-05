use crate::debug::DebugTypeName;
use crate::nut::Nut;
use crate::*;
use core::any::{Any, TypeId};

pub(crate) struct BroadcastInfo {
    address: BroadcastAddress,
    msg: Box<dyn Any>,
    topic: Topic,
    #[allow(dead_code)]
    type_name: DebugTypeName,
}

enum BroadcastAddress {
    Local(UncheckedActivityId),
    LocalByType(TypeId),
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
    pub(super) fn local_by_type<RECV: Any, MSG: Any>(msg: MSG, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::LocalByType(TypeId::of::<RECV>()),
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
                    for sub in handlers.shared_subscriptions() {
                        #[cfg(debug_assertions)]
                        self.active_activity_name.set(Some(sub.type_name));
                        let f = &sub.handler;
                        f(&mut self.activities.borrow_mut(), &mut managed_state);
                    }
                }
                BroadcastAddress::Local(id) => {
                    for sub in handlers.shared_subscriptions_of_single_activity(id) {
                        #[cfg(debug_assertions)]
                        self.active_activity_name.set(Some(sub.type_name));
                        let f = &sub.handler;
                        f(&mut self.activities.borrow_mut(), &mut managed_state);
                    }
                }
                BroadcastAddress::LocalByType(t) => {
                    let maybe_id = self.activities.borrow().id_lookup(t);
                    if let Some(id) = maybe_id {
                        if let Some(sub) = handlers.private_subscription(id) {
                            #[cfg(debug_assertions)]
                            self.active_activity_name.set(Some(sub.type_name));
                            let f = &sub.handler;
                            f(&mut self.activities.borrow_mut(), &mut managed_state);
                        } else {
                            panic!("Activity doesn't have private listener")
                        }
                    } else {
                        panic!("Activity doesn't exist")
                    }
                }
            }
            #[cfg(debug_assertions)]
            self.active_activity_name.set(None);
        }
        managed_state.clear_broadcast();
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for BroadcastInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.address {
            BroadcastAddress::Global => write!(f, "published message of type {:?}", self.type_name),
            BroadcastAddress::Local(_) => write!(f, "{:?} event", self.topic),
            BroadcastAddress::LocalByType(_) => {
                write!(f, "message of type {:?} (sent privately)", self.type_name)
            }
        }
    }
}
