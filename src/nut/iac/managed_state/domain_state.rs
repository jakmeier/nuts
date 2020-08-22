use core::any::{Any, TypeId};
use std::collections::HashMap;

/// Stores passive data that can be accessed in event handlers of multiple activities.
///
// @ START-DOC DOMAIN
/// Domains span multiple [Activities](trait.Activity.html) and allows them to share state.
/// Library users can define the domains using [`domain_enum!`](macro.domain_enum.html) but each activity can only join one domain.
///
/// If data is only used by a single activity, it is usually better to store it in the activity struct itself.
/// Domains should only be used when data needs to be shared between multiple activities of the same or different types.
///
/// For now, there is no real benefit from using multiple Domains, other than data isolation.
/// But there are plans for the future that will schedule Activities in different threads, based on their domain.
// @ END-DOC DOMAIN
#[derive(Default)]
pub struct DomainState {
    objects: HashMap<TypeId, Box<dyn Any>>,
}

impl DomainState {
    /// Stores a value in the domain.
    /// If an old value of the same type already exists in the domain, it will be overwritten.
    pub fn store<T: Any>(&mut self, obj: T) {
        self.objects.insert(TypeId::of::<T>(), Box::new(obj));
    }
    /// Returns a reference to a value of the specified type, if such a value has previously been stored to the domain.
    #[allow(clippy::unwrap_used)]
    pub fn try_get<T: Any>(&self) -> Option<&T> {
        self.objects
            .get(&TypeId::of::<T>())
            .map(|obj| obj.as_ref().downcast_ref().unwrap())
    }
    /// Same as [`try_get`](#try_get) but grants mutable access to the object.
    #[allow(clippy::unwrap_used)]
    pub fn try_get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.objects
            .get_mut(&TypeId::of::<T>())
            .map(|obj| obj.as_mut().downcast_mut().unwrap())
    }
    /// Returns a reference to a value of the specified type, taken from the domain.
    /// # Panics
    /// Panics if object of that type has not been stored previously.
    /// [`try_get()`](#try_get) is usually recommended instead.
    #[allow(clippy::unwrap_used)]
    pub fn get<T: Any>(&self) -> &T {
        self.objects
            .get(&TypeId::of::<T>())
            .map(|obj| obj.as_ref().downcast_ref().unwrap())
            .expect("Not in domain")
    }
    /// Returns a mutable reference to a value of the specified type, taken from the domain.
    /// # Panics
    /// Panics if object of that type has not been stored previously
    /// [`try_get_mut()`](#try_get_mut) is usually recommended instead.
    #[allow(clippy::unwrap_used)]
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.objects
            .get_mut(&TypeId::of::<T>())
            .map(|obj| obj.as_mut().downcast_mut().unwrap())
            .expect("Not in domain")
    }
}
