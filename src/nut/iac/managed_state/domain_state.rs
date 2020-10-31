use core::any::{Any, TypeId};
use std::collections::{hash_map::Entry, HashMap};

/// Stores passive data that can be accessed in event handlers of multiple activities.
///
// @ START-DOC DOMAIN
/// A Domain stores arbitrary data for sharing between multiple [Activities](trait.Activity.html).
/// Library users can define the number of domains but each activity can only join one domain.
///
/// Domains should only be used when data needs to be shared between multiple activities of the same or different types.
/// If data is only used by a single activity, it is usually better to store it in the activity struct itself.
///
/// In case only one domain is used, you can also consider to use [`DefaultDomain`](struct.DefaultDomain.html) instead of creating your own enum.
///
/// For now, there is no real benefit from using multiple Domains, other than data isolation.
/// But there are plans for the future that will schedule Activities in different threads, based on their domain.
// @ END-DOC DOMAIN
#[derive(Default)]
pub struct DomainState {
    // Indirection to Vec is used here to allow for safe internal mutability without falling back to RefCells.
    // (RefCells are uneasy to use from outside AND the runtime hit is larger)
    objects: Vec<Box<dyn Any>>,
    index_map: HashMap<TypeId, usize>,
}

impl DomainState {
    /// Stores a value in the domain.
    // @ START-DOC DOMAIN_STORE
    /// Only one instance per type id can be stored inside a domain.
    /// If an old value of the same type already exists in the domain, it will be overwritten.
    // @ END-DOC DOMAIN_STORE
    pub fn store<T: Any>(&mut self, obj: T) {
        let id = TypeId::of::<T>();
        match self.index_map.entry(id) {
            Entry::Occupied(entry) => {
                *self.objects[*entry.get()].downcast_mut().unwrap() = obj;
            }
            Entry::Vacant(entry) => {
                entry.insert(self.objects.len());
                self.objects.push(Box::new(obj));
            }
        }
    }
    /// For internal use only.
    ///
    /// Non-generic variant of store.
    /// Used for delayed stores to domains.
    /// 
    /// This variant is slightly less efficient as it will allocate another Box if the value was already in the domain.
    pub(crate) fn store_unchecked(&mut self, id: TypeId, obj: Box<dyn Any>) {
        match self.index_map.entry(id) {
            Entry::Occupied(entry) => {
                self.objects[*entry.get()] = obj;
            }
            Entry::Vacant(entry) => {
                entry.insert(self.objects.len());
                self.objects.push(obj);
            }
        }
    }
    /// Returns a reference to a value of the specified type, if such a value has previously been stored to the domain.
    #[allow(clippy::unwrap_used)]
    pub fn try_get<T: Any>(&self) -> Option<&T> {
        self.index_map
            .get(&TypeId::of::<T>())
            .map(|index| self.objects[*index].as_ref().downcast_ref().unwrap())
    }
    /// Same as [`try_get`](#try_get) but grants mutable access to the object.
    #[allow(clippy::unwrap_used)]
    pub fn try_get_mut<T: Any>(&mut self) -> Option<&mut T> {
        if let Some(index) = self.index_map.get(&TypeId::of::<T>()) {
            Some(self.objects[*index].as_mut().downcast_mut().unwrap())
        } else {
            None
        }
    }
    /// Return two mutable references to domain objects
    #[allow(clippy::unwrap_used)]
    pub fn try_get_2_mut<T1: Any, T2: Any>(&mut self) -> (Option<&mut T1>, Option<&mut T2>) {
        let type_1: TypeId = TypeId::of::<T1>();
        let type_2: TypeId = TypeId::of::<T2>();
        assert_ne(type_1, type_2);
        let i1 = self.index_map.get(&type_1);
        let i2 = self.index_map.get(&type_2);
        if i1.is_none() {
            return (None, self.try_get_mut());
        }
        if i2.is_none() {
            return (self.try_get_mut(), None);
        }

        let i1 = i1.unwrap();
        let i2 = i2.unwrap();

        let split = i1.min(i2) + 1;
        let (left, right) = self.objects.split_at_mut(split);

        let (t1, t2) = if i1 < i2 {
            (&mut left[*i1], &mut right[i2 - split])
        } else {
            (&mut right[i1 - split], &mut left[*i2])
        };
        (
            Some(t1.as_mut().downcast_mut().unwrap()),
            Some(t2.as_mut().downcast_mut().unwrap()),
        )
    }
    /// Returns a reference to a value of the specified type, taken from the domain.
    /// # Panics
    /// Panics if object of that type has not been stored previously.
    /// [`try_get()`](#try_get) is usually recommended instead.
    #[allow(clippy::unwrap_used)]
    pub fn get<T: Any>(&self) -> &T {
        self.try_get().expect("Not in domain")
    }
    /// Returns a mutable reference to a value of the specified type, taken from the domain.
    /// # Panics
    /// Panics if object of that type has not been stored previously
    /// [`try_get_mut()`](#try_get_mut) is usually recommended instead.
    #[allow(clippy::unwrap_used)]
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.try_get_mut().expect("Not in domain")
    }
}
// This should really be a const fn so that we get compile-time panic instead of run-time checks.
// But unfortunately, that is currently not possible.
fn assert_ne(t1: TypeId, t2: TypeId) {
    if t1 == t2 {
        panic!("Cannot get two mutable references of same type from domain")
    }
}
