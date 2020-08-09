//! Top-level module for all the inner-magic of nuts.
//!
//! Nothing in here is public interface but documentation is still important for
//! library developers as well as users if they want to understand more how this library works.

pub(crate) mod activity;
pub(crate) mod exec;
pub(crate) mod iac;

#[cfg(test)]
mod test;

use crate::nut::exec::Deferred;
use crate::*;
use core::any::Any;
use core::sync::atomic::AtomicBool;
use exec::fifo::ThreadLocalFifo;
use iac::managed_state::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local!(static NUT: Nut = Nut::new());

/// A nut stores thread-local state and provides an easy interface to access it.
///
/// To allow nested access to the nut, it is a read-only structure.
/// The field of it can be accessed separately. The library is designed carefully to
/// ensure single-write/multiple-reader is enforced at all times.
/// (As the API matures, user-side errors should become more and more unlikely)
#[derive(Default)]
struct Nut {
    /// Stores the data for activities, the semi-isolated components of this library.
    /// Mutable access given atomically on each closure dispatch.
    activities: RefCell<ActivityContainer>,
    /// Keeps state necessary for inter-activity communication. (domain state and message slot)
    /// Mutable access given atomically on each closure dispatch.
    managed_state: RefCell<ManagedState>,
    /// Closures sorted by topic.
    /// Mutable access only from outside of handlers, preferably before first publish call.
    /// Read-only access afterwards.
    /// (This restriction might change in the future)
    subscriptions: RefCell<HashMap<Topic, ActivityHandlerContainer>>,
    /// FIFO queue for published messages and other events that cannot be processed immediately.
    /// Atomically accessed mutably between closure dispatches.
    deferred_events: ThreadLocalFifo<Deferred>,
    /// A flag that marks if a broadcast is currently on-going
    executing: AtomicBool,
}

/// A method that can be called by the ActivityManager.
/// These handlers are created by the library and not part of the public interface.
type Handler = Box<dyn Fn(&mut ActivityContainer, &mut ManagedState)>;

impl Nut {
    fn new() -> Self {
        Default::default()
    }
    fn push_closure<A: 'static>(&self, topic: Topic, id: ActivityId<A>, closure: Handler) {
        self.subscriptions
            .try_borrow_mut()
            .expect("Tried to add a new listener from inside a listener, which is not allowed.")
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
        let err = "Adding new activities from inside an activity is not allowed.";
        nut.managed_state
            .try_borrow_mut()
            .expect(err)
            .prepare(domain_index);
        nut.activities
            .try_borrow_mut()
            .expect(err)
            .add(activity, domain_index, start_active)
    })
}

pub(crate) fn publish_custom<A: Any>(a: A) {
    NUT.with(|nut| nut.publish(a))
}

pub(crate) fn register<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_closure::<_, _, MSG>(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}
pub(crate) fn register_mut<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &mut MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_closure_mut::<_, _, MSG>(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}
pub(crate) fn register_owned<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_closure_owned::<_, _, MSG>(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}

/// For subscriptions without payload
pub(crate) fn register_no_payload<A, F>(
    id: ActivityId<A>,
    f: F,
    topic: Topic,
    filter: SubscriptionFilter,
) where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_closure::<_, _, ()>(move |a, ()| f(a), id, filter);
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
        let closure = ManagedState::pack_domained_closure(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}
pub(crate) fn register_domained_mut<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &mut DomainState, &mut MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_domained_closure_mut(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}
pub(crate) fn register_domained_owned<A, F, MSG>(
    id: ActivityId<A>,
    f: F,
    filter: SubscriptionFilter,
) where
    A: Activity,
    F: Fn(&mut A, &mut DomainState, MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_domained_closure_owned(f, id, filter);
        let topic = Topic::message::<MSG>();
        nut.push_closure(topic, id, closure);
    });
}

/// For subscriptions without payload but with domain access
pub(crate) fn register_domained_no_payload<A, F>(
    id: ActivityId<A>,
    f: F,
    topic: Topic,
    filter: SubscriptionFilter,
) where
    A: Activity,
    F: Fn(&mut A, &mut DomainState) + 'static,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_domained_closure(move |a, d, ()| f(a, d), id, filter);
        nut.push_closure(topic, id, closure);
    });
}

pub(crate) fn set_active(id: UncheckedActivityId, active: bool) {
    NUT.with(|nut| nut.set_active(id, active));
}

pub(crate) fn write_domain<D, T>(domain: D, data: T) -> Result<(), std::cell::BorrowMutError>
where
    D: DomainEnumeration,
    T: core::any::Any,
{
    NUT.with(|nut| {
        let id = DomainId::new(domain);
        let mut managed_state = nut.managed_state.try_borrow_mut()?;
        managed_state.prepare(id);
        let storage = managed_state.get_mut(id).expect("No domain");
        storage.store(data);
        Ok(())
    })
}
