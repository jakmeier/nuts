use core::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct DomainState {
    objects: HashMap<TypeId, Box<dyn Any>>,
}

impl DomainState {
    pub fn store<T: Any>(&mut self, obj: T) {
        self.objects.insert(TypeId::of::<T>(), Box::new(obj));
    }
    pub fn try_get<T: Any>(&self) -> Option<&T> {
        self.objects
            .get(&TypeId::of::<T>())
            .map(|obj| obj.as_ref().downcast_ref().unwrap())
    }
    pub fn try_get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.objects
            .get_mut(&TypeId::of::<T>())
            .map(|obj| obj.as_mut().downcast_mut().unwrap())
    }
    /// # Panics
    /// Panics if object of that type has not been stored previously.
    /// try_get() is usually recommended instead.
    pub fn get<T: Any>(&self) -> &T {
        self.objects
            .get(&TypeId::of::<T>())
            .map(|obj| obj.as_ref().downcast_ref().unwrap())
            .expect("Not in domain")
    }
    /// # Panics
    /// Panics if object of that type has not been stored previously
    /// try_get_mut() is usually recommended instead.
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.objects
            .get_mut(&TypeId::of::<T>())
            .map(|obj| obj.as_mut().downcast_mut().unwrap())
            .expect("Not in domain")
    }
}
