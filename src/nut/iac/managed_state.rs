//! Managed State
//!
//! Objects to which multiple activities have access

mod domain_id;
mod domain_state;

pub use domain_id::*;
pub use domain_state::*;

#[derive(Default)]
pub(crate) struct ManagedState {
    domains: Vec<DomainState>,
}

impl ManagedState {
    pub(crate) fn get_mut(&mut self, id: DomainId) -> Option<&mut DomainState> {
        id.index().map(move |i| &mut self.domains[i])
    }
    pub(crate) fn prepare(&mut self, id: DomainId) {
        if let Some(n) = id.index() {
            while self.domains.len() <= n {
                self.domains.push(Default::default());
            }
        }
    }
}
