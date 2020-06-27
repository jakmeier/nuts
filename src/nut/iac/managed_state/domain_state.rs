use std::collections::HashMap;
use core::any::{Any, TypeId};

#[derive(Default)]
pub struct DomainState {
    objects: HashMap<TypeId, Box<dyn Any>>,
}

impl DomainState {
    pub fn store<T: Any>(&mut self, obj: T) {
        self.objects.insert(TypeId::of::<T>(), Box::new(obj));
    }
    pub fn get<T: Any>(&self) -> &T {
        self.objects
            .get(&TypeId::of::<T>())
            .map(|obj| obj.as_ref().downcast_ref().unwrap())
            .expect("Not in domain")
    }
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.objects
            .get_mut(&TypeId::of::<T>())
            .map(|obj| obj.as_mut().downcast_mut().unwrap())
            .expect("Not in domain")
    }
}
