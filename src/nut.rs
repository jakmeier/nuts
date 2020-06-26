pub(crate) mod activity;
pub(crate) mod iac;

#[cfg(test)]
mod test;

use crate::*;
use iac::managed_state::*;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local!(static NUT: RefCell<Nut> = RefCell::new(Nut::new()));

/// The ActivityManager
#[derive(Default)]
struct Nut {
    activities: ActivityContainer,
    managed_state: ManagedState,
    subscriptions: HashMap<Topic, ActivityHandlerContainer>,
}

/// A method that can be called by the ActivityManager.
/// These handlers are created by the library and not part of the public interface.
type Handler = Box<dyn Fn(&mut ActivityContainer, &mut ManagedState)>;

impl Nut {
    fn new() -> Self {
        Default::default()
    }
    fn push_closure<A: 'static>(&mut self, topic: Topic, id: ActivityId<A>, closure: Handler) {
        self.subscriptions
            .entry(topic)
            .or_insert_with(Default::default)[id]
            .push(closure);
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

pub(crate) fn publish_custom<A: Any>(a: A) {
    NUT.with(|nut| nut.borrow_mut().publish(a))
}

pub(crate) fn register<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let closure = ManagedState::pack_closure::<_, _, _, MSG>(f, id, filter);
        let topic = Topic::custom::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}

/// For subscriptions without payload
pub(crate) fn register_no_payload<A, F>(id: ActivityId<A>, f: F, topic: Topic)
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let closure =
            ManagedState::pack_closure::<_, _, _, ()>(move |a, ()| f(a), id, Default::default());
        nut.push_closure(topic, id, closure);
    });
}

pub(crate) fn register_domained<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &mut DomainState, &MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let closure = ManagedState::pack_domained_closure(f, id, filter);
        let topic = Topic::custom::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}

pub(crate) fn set_active<A: Activity>(id: ActivityId<A>, active: bool) {
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let before = nut.activities.is_active(id);
        println!("before: {:?}, active: {:?}", before, active);
        if before != active {
            // Needs to be called before setting active, otherwise the active filter would mask the call
            if !active {
                nut.publish_local(id, Topic::leave(), ());
            }
            nut.activities.set_active(id, active);
            if active {
                nut.publish_local(id, Topic::enter(), ());
            }
        }
    });
}

pub(crate) fn write_domain<D, T>(domain: D, data: T)
where
    D: DomainEnumeration,
    T: std::any::Any,
{
    NUT.with(|nut| {
        let id = DomainId::new(domain);
        let mut nut = nut.borrow_mut();
        nut.managed_state.prepare(id);
        let storage = nut.managed_state.get_mut(id).expect("No domain");
        storage.store(data)
    })
}
