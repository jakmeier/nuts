//! Managed State
//!
//! Objects to which multiple activities have access

mod domain_id;
mod domain_state;
mod domain_store;

use crate::nut::activity::Activity;
use crate::nut::activity::ActivityContainer;
use crate::nut::activity::ActivityId;
use crate::nut::iac::filter::SubscriptionFilter;
use crate::nut::Handler;
use crate::nut::IMPOSSIBLE_ERR_MSG;
use core::any::Any;
pub use domain_id::*;
pub use domain_state::*;
pub(crate) use domain_store::*;

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
            .expect(IMPOSSIBLE_ERR_MSG)
            .downcast_mut()
            .expect(IMPOSSIBLE_ERR_MSG);
        msg
    }
    fn current_broadcast_and_domain<A: Any>(&mut self, id: DomainId) -> (&mut A, &mut DomainState) {
        let msg: &mut A = self
            .broadcast
            .as_mut()
            .expect(IMPOSSIBLE_ERR_MSG)
            .downcast_mut()
            .expect(IMPOSSIBLE_ERR_MSG);
        let i = id.index().expect(IMPOSSIBLE_ERR_MSG);
        let domain = &mut self.domains[i];
        (msg, domain)
    }
    fn take_current_broadcast<A: Any>(&mut self) -> Box<A> {
        let msg = self
            .broadcast
            .take()
            .expect(IMPOSSIBLE_ERR_MSG)
            .downcast()
            .expect(IMPOSSIBLE_ERR_MSG);
        msg
    }
    fn take_current_broadcast_and_borrow_domain<A: Any>(
        &mut self,
        id: DomainId,
    ) -> (Box<A>, &mut DomainState) {
        let msg = self
            .broadcast
            .take()
            .expect(IMPOSSIBLE_ERR_MSG)
            .downcast()
            .expect(IMPOSSIBLE_ERR_MSG);
        let i = id.index().expect("Activity has no domain");
        let domain = &mut self.domains[i];
        (msg, domain)
    }

    pub(crate) fn pack_closure_no_payload<A, F>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A) + 'static,
    {
        Box::new(
            move |activities: &mut ActivityContainer, _: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect(IMPOSSIBLE_ERR_MSG);
                    f(a)
                }
            },
        )
    }

    pub(crate) fn pack_closure_domained_no_payload<A, F>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &mut DomainState) + 'static,
    {
        Box::new(
            move |activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                if activities.filter(index, &filter) {
                    let a = activities[index]
                        .downcast_mut::<A>()
                        .expect(IMPOSSIBLE_ERR_MSG);
                    let domain = &mut managed_state.domains
                        [index.domain_index.index().expect(IMPOSSIBLE_ERR_MSG)];
                    f(a, domain)
                }
            },
        )
    }

    pub(crate) fn pack_closure_no_activity<F, MSG>(f: F) -> Handler
    where
        F: Fn(&MSG) + 'static,
        MSG: Any,
    {
        Box::new(
            move |_activities: &mut ActivityContainer, managed_state: &mut ManagedState| {
                let msg = managed_state.current_broadcast();
                f(msg)
            },
        )
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
                        .expect(IMPOSSIBLE_ERR_MSG);
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
                        .expect(IMPOSSIBLE_ERR_MSG);
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
                        .expect(IMPOSSIBLE_ERR_MSG);
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
                        .expect(IMPOSSIBLE_ERR_MSG);
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
                        .expect(IMPOSSIBLE_ERR_MSG);
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
                        .expect(IMPOSSIBLE_ERR_MSG);
                    let (msg, domain) =
                        managed_state.take_current_broadcast_and_borrow_domain(index.domain_index);
                    f(a, domain, *msg)
                }
            },
        )
    }
}
