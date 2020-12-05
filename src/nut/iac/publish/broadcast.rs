use crate::debug::DebugTypeName;
use crate::nut::{iac::subscription::Subscription, Nut};
use crate::*;
use core::any::{Any, TypeId};
use std::cell::RefMut;

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
    pub(crate) fn global<MSG: Any>(msg: MSG, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Global,
            msg: Box::new(msg),
            topic,
            type_name: DebugTypeName::new::<MSG>(),
        }
    }
    pub(crate) fn local<MSG: Any>(msg: MSG, id: UncheckedActivityId, topic: Topic) -> Self {
        BroadcastInfo {
            address: BroadcastAddress::Local(id),
            msg: Box::new(msg),
            topic,
            type_name: DebugTypeName::new::<MSG>(),
        }
    }
    pub(crate) fn local_by_type<RECV: Any, MSG: Any>(msg: MSG, topic: Topic) -> Self {
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
            match self.receiver_id(&broadcast.address) {
                None => {
                    for sub in handlers.shared_subscriptions() {
                        self.call_subscriber(sub, &mut managed_state);
                    }
                }
                Some(id) => {
                    if broadcast.topic.unqiue_per_activity() {
                        if let Some(sub) = handlers.private_subscription(id) {
                            self.call_subscriber(sub, &mut managed_state);
                        }
                    } else {
                        for sub in handlers.shared_subscriptions_of_single_activity(id) {
                            self.call_subscriber(sub, &mut managed_state);
                        }
                    }
                }
            }
            #[cfg(debug_assertions)]
            self.active_activity_name.set(None);
        }
        managed_state.clear_broadcast();
    }
    fn call_subscriber(&self, sub: &Subscription, managed_state: &mut RefMut<ManagedState>) {
        #[cfg(debug_assertions)]
        self.active_activity_name.set(Some(sub.type_name));
        let f = &sub.handler;
        f(&mut self.activities.borrow_mut(), managed_state);
    }
    fn receiver_id(&self, address: &BroadcastAddress) -> Option<UncheckedActivityId> {
        match address {
            BroadcastAddress::Global => None,
            BroadcastAddress::Local(id) => Some(*id),
            BroadcastAddress::LocalByType(t) => self.activities.borrow().id_lookup(*t),
        }
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
