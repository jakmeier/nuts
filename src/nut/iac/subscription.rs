use super::topic::Topic;
use crate::ActivityHandlerContainer;
use crate::{
    nut::{Handler, IMPOSSIBLE_ERR_MSG},
    UncheckedActivityId,
};
use core::cell::Ref;
use std::{cell::RefCell, collections::HashMap};

#[derive(Default)]
pub(crate) struct Subscriptions {
    subscriptions: RefCell<HashMap<Topic, ActivityHandlerContainer>>,
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
