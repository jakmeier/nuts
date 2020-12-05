pub(crate) use broadcast::BroadcastInfo;

mod broadcast;
mod response;
pub(crate) use response::ResponseTracker;
pub(crate) use response::Slot as ResponseSlot;

use crate::nut::Nut;
use crate::*;
use core::any::Any;

use self::response::NutsResponse;

impl Nut {
    pub(crate) fn broadcast(&self, broadcast: BroadcastInfo) {
        self.deferred_events.push(broadcast.into());
        self.catch_up_deferred_to_quiescence();
    }
    pub(crate) fn publish_and_await<MSG: Any>(&self, msg: MSG) -> NutsResponse {
        let broadcast = BroadcastInfo::global(msg, Topic::public_message::<MSG>());
        let ticket = Nut::with_response_tracker_mut(|rt| rt.allocate());
        let future = NutsResponse::new(&ticket);
        self.deferred_events
            .push(nut::exec::Deferred::BroadcastAwaitingResponse(
                broadcast, ticket,
            ));
        self.catch_up_deferred_to_quiescence();
        future
    }
}
