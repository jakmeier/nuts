use crate::activity::*;
use crate::publish::*;
use crate::topic::*;
use crate::SubscriptionFilter;
use std::cell::RefCell;

thread_local!(static NUT: RefCell<Nut> = RefCell::new(Nut::new()));

/// The ActivityManager
#[derive(Default)]
pub(crate) struct Nut {
    pub(crate) activities: ActivityContainer,
    pub(crate) updates: Vec<Handler>,
    pub(crate) draw: Vec<Handler>,
    pub(crate) enter: ActivityHandlerContainer,
    pub(crate) leave: ActivityHandlerContainer,
    // TODO: listen-event
}

/// A method that can be called by the ActivityManager.
/// These handlers are created by the library and not part of the public interface.
pub(crate) type Handler = Box<dyn Fn(&mut ActivityContainer)>;

impl Nut {
    fn new() -> Self {
        Default::default()
    }
}

pub(crate) fn new_activity<A>(activity: A, start_active: bool) -> ActivityId<A>
where
    A: Activity,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        nut.activities.add(activity, start_active)
    })
}

pub(crate) fn publish_builtin(topic: GlobalNotification) {
    NUT.with(|nut| nut.borrow_mut().publish_global(topic))
}

pub(crate) fn register<A, F>(id: ActivityId<A>, topic: Topic, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let closure = pack_closure(f, id, filter);
        match topic {
            Topic::Builtin(BuiltinTopic::Update) => {
                nut.updates.push(closure);
            }
            Topic::Builtin(BuiltinTopic::Draw) => {
                nut.draw.push(closure);
            }
            Topic::Builtin(BuiltinTopic::Enter) => {
                nut.enter[id].push(closure);
            }
            Topic::Builtin(BuiltinTopic::Leave) => {
                nut.leave[id].push(closure);
            }
        }
    });
}

fn pack_closure<A, F>(f: F, index: ActivityId<A>, filter: SubscriptionFilter) -> Handler
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    Box::new(move |activities: &mut ActivityContainer| {
        if activities.filter(index, &filter) {
            let a = activities[index]
                .downcast_mut::<A>()
                .expect("Wrong activity"); // deleted and replaced?
            f(a)
        }
    })
}

pub(crate) fn set_active<A: Activity>(id: ActivityId<A>, active: bool) {
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let before = nut.activities.is_active(id);
        println!("before: {:?}, active: {:?}", before, active);
        if before != active {
            // Needs to be called before setting active, otherwise the active filter would mask the call
            if !active {
                nut.publish(id, LocalNotification::Leave);
            }
            nut.activities.set_active(id, active);
            if active {
                nut.publish(id, LocalNotification::Enter);
            }
        }
    });
}
