pub(crate) use broadcast::BroadcastInfo;

mod broadcast;

use crate::nut::Nut;
use crate::*;
use core::any::Any;

impl Nut {
    pub(crate) fn publish_local<MSG: Any>(&self, id: UncheckedActivityId, topic: Topic, msg: MSG) {
        let broadcast = BroadcastInfo::local(msg, id, topic);
        self.deferred_events.push(broadcast.into());
        self.catch_up_deferred_to_quiescence();
    }
    pub(crate) fn publish<MSG: Any>(&self, msg: MSG) {
        let broadcast = BroadcastInfo::global(msg, Topic::message::<MSG>());
        self.deferred_events.push(broadcast.into());
        self.catch_up_deferred_to_quiescence();
    }
}
