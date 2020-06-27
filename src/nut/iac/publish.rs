use crate::nut::Nut;
use crate::*;
use core::any::Any;

impl Nut {
    pub fn publish_local<A: Activity, MSG: Any>(
        &mut self,
        id: ActivityId<A>,
        topic: Topic,
        a: MSG,
    ) {
        self.managed_state.push_broadcast(a);
        if let Some(handlers) = self.subscriptions.get(&topic) {
            for f in handlers.iter_for(id) {
                f(&mut self.activities, &mut self.managed_state);
            }
        }
        self.managed_state.end_current_broadcast();
    }
    pub fn publish<MSG: Any>(&mut self, a: MSG) {
        self.managed_state.push_broadcast(a);
        let topic = Topic::message::<MSG>();
        if let Some(handlers) = self.subscriptions.get(&topic) {
            for f in handlers.iter() {
                f(&mut self.activities, &mut self.managed_state);
            }
        }
        self.managed_state.end_current_broadcast();
    }
}
