use crate::nut::Nut;
use crate::*;

pub(crate) enum GlobalNotification {
    Draw,
    Update,
}

pub(crate) enum LocalNotification {
    Enter,
    Leave,
}

impl Nut {
    pub fn publish_global(&mut self, topic: GlobalNotification) {
        let handlers = match topic {
            GlobalNotification::Draw => &self.draw,
            GlobalNotification::Update => &self.updates,
        };
        for f in handlers {
            f(&mut self.activities);
        }
    }
    pub fn publish<A: Activity>(&mut self, id: ActivityId<A>, topic: LocalNotification) {
        let handlers = match topic {
            LocalNotification::Enter => &self.enter,
            LocalNotification::Leave => &self.leave,
        };
        for f in handlers.iter_for(id) {
            println!("Calling now");
            f(&mut self.activities);
        }
    }
}
