#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(crate) struct DomainId(Option<usize>);

/// Used for mapping domain identifiers to unique integers.
/// Can be derived with `domain_enum!(TYPE)`;
pub trait DomainEnumeration {
    /// The unique integer for a specific domain
    fn id(&self) -> usize;
}

/// If only one domain is required, this can be used.
/// But if you have your own domain type defined with `domain_enum!` or implementing `DomainEnumeration` manually, do not use this.
/// Different domain types should never be mixed!
pub struct DefaultDomain;

impl DomainEnumeration for DefaultDomain {
    fn id(&self) -> usize {
        0
    }
}

impl DomainId {
    pub(crate) fn new(d: &impl DomainEnumeration) -> DomainId {
        DomainId(Some(d.id()))
    }
    pub(crate) fn index(&self) -> Option<usize> {
        self.0
    }
}

#[macro_export]
/// Implements DomainEnumeration for an enum.
///
/// This macro can only be used on primitive enums that implement Copy.
/// The current implementation of the macro unfortunately also requires
/// `DomainEnumeration` to be imported with this exact name.
///
/// # Example:
// @ START-DOC DOMAIN_MACRO_EXAMPLE
/// ```
/// #[macro_use] extern crate nuts;
/// use nuts::{domain_enum, DomainEnumeration};
/// #[derive(Clone, Copy)]
/// enum MyDomain {
///     DomainA,
///     DomainB,
/// }
/// domain_enum!(MyDomain);
/// ```
// @ END-DOC DOMAIN_MACRO_EXAMPLE
macro_rules! domain_enum {
    ( $e:tt ) => {
        impl DomainEnumeration for $e {
            fn id(&self) -> usize {
                *self as usize
            }
        }
    };
}
