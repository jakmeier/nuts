use crate::debug::DebugTypeName;
use crate::nut::{Nut, IMPOSSIBLE_ERR_MSG};
use crate::DomainId;
use core::any::{Any, TypeId};

pub(crate) struct DomainStoreData {
    domain: DomainId,
    id: TypeId,
    data: Box<dyn Any>,
    #[allow(dead_code)]
    type_name: DebugTypeName,
}
impl Nut {
    pub fn exec_domain_store(&self, d: DomainStoreData) {
        self.managed_state
            .try_borrow_mut()
            .expect(IMPOSSIBLE_ERR_MSG)
            .get_mut(d.domain)
            .expect("Domain ID invalid")
            .store_unchecked(d.id, d.data);
    }
}

impl DomainStoreData {
    pub fn new<DATA: Any>(domain: DomainId, data: DATA) -> Self {
        Self {
            domain,
            id: TypeId::of::<DATA>(),
            data: Box::new(data),
            type_name: DebugTypeName::new::<DATA>(),
        }
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for DomainStoreData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storing {:?} to the domain", self.type_name)
    }
}
