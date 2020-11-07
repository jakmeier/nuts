use super::{managed_state::ManagedState, topic::Topic};
use crate::ActivityHandlerContainer;
use crate::{
    nut::{Handler, IMPOSSIBLE_ERR_MSG},
    UncheckedActivityId,
};
use core::cell::Ref;
use std::{any::Any, cell::RefCell, collections::HashMap};

#[derive(Default)]
pub(crate) struct Subscriptions {
    subscriptions: RefCell<HashMap<Topic, ActivityHandlerContainer>>,
}

pub(crate) enum OnDelete {
    None,
    Simple(Box<dyn FnOnce(Box<dyn Any>)>),
    WithDomain(Box<dyn FnOnce(Box<dyn Any>, &mut ManagedState)>),
}

impl Subscriptions {
    pub(crate) fn force_push_closure(
        &self,
        topic: Topic,
        id: impl Into<UncheckedActivityId>,
        closure: Handler,
    ) {
        self.subscriptions
            .try_borrow_mut()
            .expect(IMPOSSIBLE_ERR_MSG)
            .entry(topic)
            .or_insert_with(Default::default)[id.into()]
        .push(closure);
    }
    pub(crate) fn get(&self) -> Ref<HashMap<Topic, ActivityHandlerContainer>> {
        self.subscriptions.borrow()
    }
}
