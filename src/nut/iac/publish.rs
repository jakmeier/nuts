pub mod fifo;
pub(crate) use broadcast::BroadcastInfo;

mod broadcast;

use crate::nut::Nut;
use crate::*;
use core::any::Any;

impl Nut {
    pub(crate) fn publish_local<A: Activity, MSG: Any>(
        &self,
        id: ActivityId<A>,
        topic: Topic,
        msg: MSG,
    ) {
        let broadcast = BroadcastInfo::local(msg, id, topic);
        self.deferred_broadcasts.push(broadcast);
        self.broadcast();
    }
    pub(crate) fn publish<MSG: Any>(&self, msg: MSG) {
        let broadcast = BroadcastInfo::global(msg, Topic::message::<MSG>());
        self.deferred_broadcasts.push(broadcast);
        self.broadcast();
    }
    pub(crate) fn publish_mut<MSG: Any>(&self, _a: &mut MSG) {
        unimplemented!()
        // self.managed_state.push_broadcast(a);
        // let topic = Topic::message::<MSG>();
        // if let Some(handlers) = self.subscriptions.get(&topic) {
        //     for f in handlers.iter() {
        //         f(&mut self.activities, &mut self.managed_state);
        //     }
        // }
        // self.managed_state.end_current_broadcast();
    }
}
