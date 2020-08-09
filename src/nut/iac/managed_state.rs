//! Managed State
//!
//! Objects to which multiple activities have access

mod domain_id;
mod domain_state;

use crate::nut::activity::Activity;
use crate::nut::activity::ActivityContainer;
use crate::nut::activity::ActivityId;
use crate::nut::iac::filter::SubscriptionFilter;
use crate::nut::Handler;
use core::any::Any;
pub use domain_id::*;
pub use domain_state::*;

#[derive(Default)]
pub(crate) struct ManagedState {
    domains: Vec<DomainState>,
    broadcast: Option<Box<dyn Any>>,
}

impl ManagedState {
    pub(crate) fn get_mut(&mut self, id: DomainId) -> Option<&mut DomainState> {
        id.index().map(move |i| &mut self.domains[i])
    }
    /// Fills all domains with default values. Must be called once or will panic when used.
    pub(crate) fn prepare(&mut self, id: DomainId) {
        if let Some(n) = id.index() {
            while self.domains.len() <= n {
                self.domains.push(Default::default());
            }
        }
    }
    pub(crate) fn set_broadcast(&mut self, msg: Box<dyn Any>) {
        self.broadcast = Some(msg);
    }
    pub(crate) fn clear_broadcast(&mut self) {
        self.broadcast = None;
    }
    /// panics if runtime broadcast is not of static type A
    fn current_broadcast<A: Any>(&mut self) -> &mut A {
        let msg = self
            .broadcast
            .as_mut()
            .expect("Bug: nothing broadcasted")
            .downcast_mut()
            .expect("Bug: wrong message broadcasted");
        msg
    }
    fn current_broadcast_and_domain<A: Any>(&mut self, id: DomainId) -> (&mut A, &mut DomainState) {
        let msg: &mut A = self
            .broadcast
            .as_mut()
            .expect("Bug: nothing broadcasted")
            .downcast_mut()
            .expect("Bug: wrong message broadcasted");
        let i = id.index().expect("Activity has no domain");
        let domain = &mut self.domains[i];
        (msg, domain)
    }
    fn take_current_broadcast<A: Any>(&mut self) -> Box<A> {
        let msg = self
            .broadcast
            .take()
            .expect("Bug: nothing broadcasted")
            .downcast()
            .expect("Bug: wrong message broadcasted");
        msg
    }
    fn take_current_broadcast_and_borrow_domain<A: Any>(
        &mut self,
        id: DomainId,
    ) -> (Box<A>, &mut DomainState) {
        let msg = self
            .broadcast
            .take()
            .expect("Bug: nothing broadcasted")
            .downcast()
            .expect("Bug: wrong message broadcasted");
        let i = id.index().expect("Activity has no domain");
        let domain = &mut self.domains[i];
        (msg, domain)
    }

    pub(crate) fn pack_closure<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let msg = managed_state.current_broadcast();
                    f(a, msg)
                }
            },
        )
    }
    pub(crate) fn pack_closure_mut<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &mut MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let msg = managed_state.current_broadcast();
                    f(a, msg)
                }
            },
        )
    }
    pub(crate) fn pack_closure_owned<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let msg = managed_state.take_current_broadcast();
                    f(a, *msg)
                }
            },
        )
    }
    pub(crate) fn pack_domained_closure<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &mut DomainState, &MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let (msg, domain) =
                        managed_state.current_broadcast_and_domain(index.domain_index);
                    f(a, domain, msg)
                }
            },
        )
    }
    pub(crate) fn pack_domained_closure_mut<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &mut DomainState, &mut MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let (msg, domain) =
                        managed_state.current_broadcast_and_domain(index.domain_index);
                    f(a, domain, msg)
                }
            },
        )
    }
    pub(crate) fn pack_domained_closure_owned<A, F, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &mut DomainState, MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect("Wrong activity"); // deleted and replaced?
                    let (msg, domain) =
                        managed_state.take_current_broadcast_and_borrow_domain(index.domain_index);
                    f(a, domain, *msg)
                }
            },
        )
    }
}
