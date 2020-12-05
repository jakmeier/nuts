//! Top-level module for all the inner-magic of nuts.
//!
//! Nothing in here is public interface but documentation is still important for
//! library developers as well as users if they want to understand more how this library works.

pub(crate) mod activity;
pub(crate) mod exec;
pub(crate) mod iac;

use crate::nut::exec::Deferred;
use crate::nut::iac::subscription::OnDelete;
use crate::*;
use crate::{debug::DebugTypeName, nut::exec::inchoate::InchoateActivityContainer};
use core::any::Any;
use core::sync::atomic::AtomicBool;
use exec::fifo::ThreadLocalFifo;
use iac::managed_state::*;
use std::cell::RefCell;

use self::iac::{publish::ResponseTracker, subscription::Subscriptions};

thread_local!(static NUT: Nut = Nut::new());

pub(crate) const IMPOSSIBLE_ERR_MSG: &str =
    "Bug in nuts. It should be impossible to trigger this panic through any combinations of library calls.";

/// A nut stores thread-local state and provides an easy interface to access it.
///
/// To allow nested access to the nut, it is a read-only structure.
/// The field of it can be accessed separately. The library is designed carefully to
/// ensure single-write/multiple-reader is enforced at all times.
#[derive(Default)]
struct Nut {
    /// Stores the data for activities, the semi-isolated components of this library.
    /// Mutable access given on each closure dispatch.
    activities: RefCell<ActivityContainer>,
    /// Keeps state necessary for inter-activity communication. (domain state and message slot)
    /// Mutable access given on each closure dispatch.
    managed_state: RefCell<ManagedState>,
    /// Closures sorted by topic.
    /// Mutable access only from outside of handlers, preferably before first publish call.
    /// Read-only access afterwards.
    /// (This restriction might change in the future)
    subscriptions: Subscriptions,
    /// FIFO queue for published messages and other events that cannot be processed immediately.
    /// Atomically accessed mutably between closure dispatches.
    deferred_events: ThreadLocalFifo<Deferred>,
    /// Tracks awaited responses, which are pending futures.
    /// Used when creating new futures (NutsResponse) and when polling the same.
    /// Atomically accessed in with_response_tracker_mut() only.
    response_tracker: RefCell<ResponseTracker>,
    /// A flag that marks if a broadcast is currently on-going
    executing: AtomicBool,
    /// When executing a broadcast, `activities` and `managed_state` is not available.
    /// To still be able to add new activities during that time, temporary
    /// structures are used to buffer additions. Theses are then merged in a deferred event.
    /// (Note: Adding subscriptions does not require additional structure because they will
    /// be queued and only executed after the activity is available anyway)
    inchoate_activities: RefCell<InchoateActivityContainer>,
    /// For debugging messages
    #[cfg(debug_assertions)]
    active_activity_name: std::cell::Cell<Option<DebugTypeName>>,
}

/// A method that can be called by the `ActivityManager`.
/// These handlers are created by the library and not part of the public interface.
pub(crate) type Handler = Box<dyn Fn(&mut ActivityContainer, &mut ManagedState)>;

impl Nut {
    fn new() -> Self {
        Default::default()
    }
    fn quiescent(&self) -> bool {
        !self.executing.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn add_on_delete(&self, id: UncheckedActivityId, subscription: OnDelete) {
        if self.quiescent() {
            self.activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .add_on_delete(id, subscription);
        } else {
            self.deferred_events
                .push(Deferred::OnDeleteSubscription(id, subscription))
        }
    }
    pub(crate) fn with_response_tracker_mut<T>(f: impl FnOnce(&mut ResponseTracker) -> T) -> T {
        NUT.with(|nut| {
            let mut response_tracker = nut
                .response_tracker
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG);
            f(&mut *response_tracker)
        })
    }
}

pub(crate) fn new_activity<A>(
    activity: A,
    domain_index: DomainId,
    status: LifecycleStatus,
) -> ActivityId<A>
where
    A: Activity,
{
    NUT.with(|nut| {
        // When already executing, the state is already borrowed.
        // In that case, we have to defer creation to a quiescent state.
        // In the other case, we are guaranteed to have access.
        if !nut.executing.load(std::sync::atomic::Ordering::Relaxed) {
            // Make sure domain are allocated.
            // This is currently necessary on every new_activity call, which is a bit ugly.
            // On the other hand, performance of creating new activities is only secondary priority.
            nut.managed_state
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .prepare(domain_index);
            // Make sure that length of activities is available without locking activities.
            // Again, a bit ugly but performance is secondary in this call.
            nut.inchoate_activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .inc_offset();
            nut.activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .add(activity, domain_index, status)
        } else {
            nut.deferred_events.push(Deferred::FlushInchoateActivities);
            nut.inchoate_activities
                .try_borrow_mut()
                .expect(IMPOSSIBLE_ERR_MSG)
                .add(activity, domain_index, status)
        }
    })
}

pub(crate) fn publish_custom<A: Any>(a: A) {
    NUT.with(|nut| nut.publish(a))
}

pub(crate) fn send_custom<RECV: Any, MSG: Any>(a: MSG) {
    NUT.with(|nut| nut.send_private::<RECV, MSG>(a))
}

pub(crate) async fn publish_custom_and_await<A: Any>(a: A) {
    NUT.with(move |nut| nut.publish_and_await(a)).await;
}

pub(crate) fn register<A, F, MSG>(id: ActivityId<A>, f: F, filter: SubscriptionFilter)
where
    A: Activity,
    F: Fn(&mut A, &MSG) + 'static,
    MSG: Any,
{
    NUT.with(|nut| {
        let closure = ManagedState::pack_closure::<_, _, MSG>(f, id, filter);
        let topic = Topic::public_message::<MSG>();
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
        let topic = Topic::public_message::<MSG>();
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
        let topic = Topic::private_message::<MSG>();
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
        let closure = ManagedState::pack_closure_no_payload(f, id, filter);
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
        let topic = Topic::public_message::<MSG>();
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
        let topic = Topic::public_message::<MSG>();
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
        let topic = Topic::private_message::<MSG>();
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
        let closure = ManagedState::pack_closure_domained_no_payload(f, id, filter);
        nut.push_closure(topic, id, closure);
    });
}

pub(crate) fn register_on_delete<A, F>(id: ActivityId<A>, f: F)
where
    A: Activity,
    F: FnOnce(A) + 'static,
{
    NUT.with(|nut| {
        let closure = Box::new(|a: Box<dyn Any>| {
            let activity = a.downcast().expect(IMPOSSIBLE_ERR_MSG);
            f(*activity);
        });
        nut.add_on_delete(id.into(), OnDelete::Simple(closure));
    })
}

pub(crate) fn register_domained_on_delete<A, F>(id: ActivityId<A>, f: F)
where
    A: Activity,
    F: FnOnce(A, &mut DomainState) + 'static,
{
    NUT.with(|nut| {
        let closure = Box::new(move |a: Box<dyn Any>, managed_state: &mut ManagedState| {
            let activity = a.downcast().expect(IMPOSSIBLE_ERR_MSG);
            let domain = managed_state
                .get_mut(id.domain_index)
                .expect("missing domain");
            f(*activity, domain);
        });
        let subscription = OnDelete::WithDomain(closure);
        nut.add_on_delete(id.into(), subscription);
    })
}

pub(crate) fn set_status(id: UncheckedActivityId, status: LifecycleStatus) {
    NUT.with(|nut| nut.set_status(id, status));
}

pub(crate) fn write_domain<D, T>(domain: &D, data: T)
where
    D: DomainEnumeration,
    T: core::any::Any,
{
    NUT.with(|nut| {
        let id = DomainId::new(domain);
        if let Ok(mut managed_state) = nut.managed_state.try_borrow_mut() {
            managed_state.prepare(id);
            let storage = managed_state.get_mut(id).expect("No domain");
            storage.store(data);
        } else {
            let event = Deferred::DomainStore(DomainStoreData::new(id, data));
            nut.deferred_events.push(event);
        }
    })
}

#[cfg(debug_assertions)]
pub(crate) fn nuts_panic_info() -> Option<String> {
    NUT.try_with(|nut| {
        let mut info = String::new();
        info.push_str("NUTS panic hook: Panicked while ");
        if let Some(activity) = nut.active_activity_name.get() {
            info.push_str(activity.0);
        } else {
            info.push_str("no");
        }
        info.push_str(" activity was active.\n");
        info
    })
    .ok()
}
