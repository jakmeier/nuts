use super::{managed_state::ManagedState, topic::Topic};
use crate::{
    debug::DebugTypeName,
    nut::{exec::Deferred, Handler, Nut, IMPOSSIBLE_ERR_MSG},
    ActivityId, UncheckedActivityId,
};
use core::cell::Ref;
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    ops::{Index, IndexMut},
};

#[derive(Default)]
pub(crate) struct Subscriptions {
    subscriptions: RefCell<HashMap<Topic, SubscriptionContainer>>,
}

/// Handlers stored per Activity
#[derive(Default)]
pub(crate) struct SubscriptionContainer {
    data: HashMap<usize, ActivityTopicSubscriptions>,
}

/// Handlers per type per activity
#[derive(Default)]
pub(crate) struct ActivityTopicSubscriptions {
    shared: Vec<Subscription>,
    private: Option<Subscription>,
}

pub(crate) struct Subscription {
    pub(crate) handler: Handler,
    #[allow(dead_code)]
    pub(crate) type_name: DebugTypeName,
}

pub(crate) enum OnDelete {
    None,
    Simple(Box<dyn FnOnce(Box<dyn Any>)>),
    WithDomain(Box<dyn FnOnce(Box<dyn Any>, &mut ManagedState)>),
}

impl Nut {
    pub(crate) fn push_closure<A: 'static>(
        &self,
        topic: Topic,
        id: ActivityId<A>,
        closure: Handler,
    ) {
        let type_name = DebugTypeName::new::<A>();
        if self.quiescent() {
            self.subscriptions
                .force_push_closure(topic, id, closure, type_name);
        } else {
            let sub = NewSubscription::new(topic, id, closure, type_name);
            self.deferred_events.push(Deferred::Subscription(sub));
        }
    }
}

impl Subscriptions {
    pub(crate) fn exec_new_subscription(&self, sub: NewSubscription) {
        self.force_push_closure(sub.topic, sub.id, sub.closure, sub.type_name);
    }
    fn force_push_closure(
        &self,
        topic: Topic,
        id: impl Into<UncheckedActivityId>,
        handler: Handler,
        type_name: DebugTypeName,
    ) {
        let id = id.into();
        let private = topic.unqiue_per_activity();
        let subs = &mut self
            .subscriptions
            .try_borrow_mut()
            .expect(IMPOSSIBLE_ERR_MSG);
        let subs_per_activity = &mut subs.entry(topic).or_insert_with(Default::default)[id];

        if private {
            subs_per_activity.private = Some(Subscription { handler, type_name });
        } else {
            subs_per_activity
                .shared
                .push(Subscription { handler, type_name });
        }
    }
    pub(crate) fn get(&self) -> Ref<HashMap<Topic, SubscriptionContainer>> {
        self.subscriptions.borrow()
    }
}

impl SubscriptionContainer {
    pub fn shared_subscriptions(&self) -> impl Iterator<Item = &Subscription> {
        self.data.values().flat_map(|f| f.shared.iter())
    }
    pub fn shared_subscriptions_of_single_activity(
        &self,
        id: UncheckedActivityId,
    ) -> impl Iterator<Item = &Subscription> {
        self.data
            .get(&id.index)
            .into_iter()
            .flat_map(|f| f.shared.iter())
    }
    pub fn private_subscription(&self, id: UncheckedActivityId) -> Option<&Subscription> {
        self.data
            .get(&id.index)
            .map(|f| f.private.as_ref())
            .flatten()
    }
}
impl Index<UncheckedActivityId> for SubscriptionContainer {
    type Output = ActivityTopicSubscriptions;
    fn index(&self, id: UncheckedActivityId) -> &Self::Output {
        &self.data[&id.index]
    }
}
impl IndexMut<UncheckedActivityId> for SubscriptionContainer {
    fn index_mut(&mut self, id: UncheckedActivityId) -> &mut Self::Output {
        self.data.entry(id.index).or_insert_with(Default::default)
    }
}

pub(crate) struct NewSubscription {
    topic: Topic,
    id: UncheckedActivityId,
    closure: Handler,
    type_name: DebugTypeName,
}

impl NewSubscription {
    fn new(
        topic: Topic,
        id: impl Into<UncheckedActivityId>,
        closure: Handler,
        type_name: DebugTypeName,
    ) -> Self {
        Self {
            topic,
            id: id.into(),
            closure,
            type_name,
        }
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for NewSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Adding new subscription to activity of type {:?}",
            self.type_name
        )
    }
}
