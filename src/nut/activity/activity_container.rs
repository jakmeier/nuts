use super::*;
use crate::nut::iac::subscription::OnDelete;

/// A collection of heterogenous Activities
///
/// Needs stores a list of dynamic `Any` trait objects, not `Activity` because
/// trait objects only allow access to methods of that trait, not their super-traits.  
#[derive(Default)]
pub(crate) struct ActivityContainer {
    data: Vec<Option<Box<dyn Any>>>,
    active: Vec<LifecycleStatus>,
    on_delete: Vec<OnDelete>,
}

/// Handlers stored per Activity
#[derive(Default)]
pub(crate) struct ActivityHandlerContainer {
    data: HashMap<usize, Vec<Handler>>,
}

impl ActivityContainer {
    pub(crate) fn add<A: Activity>(
        &mut self,
        a: A,
        domain: DomainId,
        status: LifecycleStatus,
    ) -> ActivityId<A> {
        let i = self.data.len();
        self.data.push(Some(Box::new(a)));
        self.active.push(status);
        self.on_delete.push(OnDelete::None);
        ActivityId::new(i, domain)
    }
    pub(crate) fn status(&self, id: UncheckedActivityId) -> LifecycleStatus {
        self.active[id.index]
    }
    pub(crate) fn set_status(&mut self, id: UncheckedActivityId, status: LifecycleStatus) {
        self.active[id.index] = status
    }
    pub(crate) fn add_on_delete(&mut self, id: UncheckedActivityId, f: OnDelete) {
        self.on_delete[id.index] = f;
    }
    pub(crate) fn delete(&mut self, id: UncheckedActivityId, managed_state: &mut ManagedState) {
        if let Some(activity) = self.data[id.index].take() {
            // Taking ownership to call FnOnce
            let mut on_delete = OnDelete::None;
            std::mem::swap(&mut on_delete, &mut self.on_delete[id.index]);
            match on_delete {
                OnDelete::None => { /* NOP  */ }
                OnDelete::Simple(f) => f(activity),
                OnDelete::WithDomain(f) => f(activity, managed_state),
            }
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        self.active.append(&mut other.active);
        self.data.append(&mut other.data);
        self.on_delete.append(&mut other.on_delete);
    }
}

impl<A: Activity> Index<ActivityId<A>> for ActivityContainer {
    type Output = dyn Any;
    fn index(&self, id: ActivityId<A>) -> &Self::Output {
        self.data[id.id.index]
            .as_ref()
            .expect("Missing activity")
            .as_ref()
    }
}
impl<A: Activity> IndexMut<ActivityId<A>> for ActivityContainer {
    fn index_mut(&mut self, id: ActivityId<A>) -> &mut Self::Output {
        self.data[id.id.index]
            .as_mut()
            .expect("Missing activity")
            .as_mut()
    }
}

impl ActivityHandlerContainer {
    pub fn iter(&self) -> impl Iterator<Item = &Handler> {
        self.data.values().flat_map(|f| f.iter())
    }
    pub fn iter_for(&self, id: UncheckedActivityId) -> impl Iterator<Item = &Handler> {
        self.data.get(&id.index).into_iter().flat_map(|f| f.iter())
    }
}
impl Index<UncheckedActivityId> for ActivityHandlerContainer {
    type Output = Vec<Handler>;
    fn index(&self, id: UncheckedActivityId) -> &Self::Output {
        &self.data[&id.index]
    }
}
impl IndexMut<UncheckedActivityId> for ActivityHandlerContainer {
    fn index_mut(&mut self, id: UncheckedActivityId) -> &mut Self::Output {
        self.data.entry(id.index).or_insert_with(Default::default)
    }
}
