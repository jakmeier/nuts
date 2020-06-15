pub(crate) mod activity;
pub(crate) mod iac;

#[cfg(test)]
mod test;

use crate::*;
use iac::managed_state::*;
use std::cell::RefCell;

thread_local!(static NUT: RefCell<Nut> = RefCell::new(Nut::new()));

/// The ActivityManager
#[derive(Default)]
struct Nut {
    activities: ActivityContainer,
    managed_state: ManagedState,
    updates: Vec<Handler>,
    draw: Vec<Handler>,
    enter: ActivityHandlerContainer,
    leave: ActivityHandlerContainer,
    // TODO: listen-event
}

/// A method that can be called by the ActivityManager.
/// These handlers are created by the library and not part of the public interface.
type Handler = Box<dyn Fn(&mut ActivityContainer, &mut ManagedState)>;

impl Nut {
    fn new() -> Self {
        Default::default()
    }
}

pub(crate) fn new_activity<A>(
    activity: A,
    domain_index: DomainId,
    start_active: bool,
) -> ActivityId<A>
where
    A: Activity,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        nut.managed_state.prepare(domain_index);
        nut.activities.add(activity, domain_index, start_active)
    })
}

pub(crate) fn publish_builtin(topic: GlobalNotification) {
    NUT.with(|nut| nut.borrow_mut().publish_global(topic))
}

pub(crate) fn register<A, F>(id: ActivityId<A>, topic: Topic, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, Option<&mut DomainState>) + 'static,
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
    F: Fn(&mut A, Option<&mut DomainState>) + 'static,
{
    Box::new(
        move |activities: &mut ActivityContainer, ms: &mut ManagedState| {
            if activities.filter(index, &filter) {
                let a = activities[index]
                    .downcast_mut::<A>()
                    .expect("Wrong activity"); // deleted and replaced?
                let domain = ms.get_mut(index.domain_index);
                f(a, domain)
            }
        },
    )
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
