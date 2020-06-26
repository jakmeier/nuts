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
pub use domain_id::*;
pub use domain_state::*;
use std::any::Any;

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
    pub(crate) fn push_broadcast<MSG: Any>(&mut self, msg: MSG) {
        self.broadcast = Some(Box::new(msg));
    }
    pub(crate) fn end_current_broadcast(&mut self) {
        self.broadcast = None;
    }
    /// panics if runtime broadcast is not of static type A
    fn current_broadcast<A: Any>(&self) -> &A {
        let msg = self
            .broadcast
            .as_ref()
            .expect("Bug: nothing broadcasted")
            .downcast_ref()
            .expect("Bug: wrong message broadcasted");
        msg
    }
    fn current_broadcast_and_domain<A: Any>(&mut self, id: DomainId) -> (&A, &mut DomainState) {
        let msg: &A = self
            .broadcast
            .as_ref()
            .expect("Bug: nothing broadcasted")
            .downcast_ref()
            .expect("Bug: wrong message broadcasted");
        let i = id.index().expect("Activity has no domain");
        let domain = &mut self.domains[i];
        (msg, domain)
    }

    pub(crate) fn pack_closure<A, F, S, MSG>(
        f: F,
        index: ActivityId<A>,
        filter: SubscriptionFilter,
    ) -> Handler
    where
        A: Activity,
        F: Fn(&mut A, &S) + 'static,
        S: Any,
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
}
