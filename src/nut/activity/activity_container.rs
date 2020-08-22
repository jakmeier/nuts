use super::*;

/// A collection of heterogenous Activities
///
/// Needs stores a list of dynamic `Any` trait objects, not `Activity` because
/// trait objects only allow access to methods of that trait, not their super-traits.  
#[derive(Default)]
pub(crate) struct ActivityContainer {
    data: Vec<Option<Box<dyn Any>>>,
    active: Vec<LifecycleStatus>,
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
        ActivityId::new(i, domain)
    }
    pub(crate) fn status(&self, id: UncheckedActivityId) -> LifecycleStatus {
        self.active[id.index]
    }
    pub(crate) fn set_status(&mut self, id: UncheckedActivityId, status: LifecycleStatus) {
        self.active[id.index] = status
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
impl<A: Activity> Index<ActivityId<A>> for ActivityHandlerContainer {
    type Output = Vec<Handler>;
    fn index(&self, id: ActivityId<A>) -> &Self::Output {
        &self.data[&id.id.index]
    }
}
impl<A: Activity> IndexMut<ActivityId<A>> for ActivityHandlerContainer {
    fn index_mut(&mut self, id: ActivityId<A>) -> &mut Self::Output {
        self.data
            .entry(id.id.index)
            .or_insert_with(Default::default)
    }
}
