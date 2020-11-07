use core::any::{Any, TypeId};

/// A topic for messages that can be published and subscribed to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Topic {
    /// Topic for a builtin event
    BuiltinEvent(BuiltinEvent),
    /// Topic for a message type, whee type is a Rust type (core::any::TypeId)
    Message(TypeId),
}

/// Builtin events are messages without payload that are used internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BuiltinEvent {
    /// On status change to active (not called if started as active)
    Enter,
    /// On status change to inactive / deleted
    Leave,
}

impl Topic {
    pub(crate) fn enter() -> Self {
        Self::BuiltinEvent(BuiltinEvent::Enter)
    }
    pub(crate) fn leave() -> Self {
        Self::BuiltinEvent(BuiltinEvent::Leave)
    }
    pub(crate) fn message<T: Any>() -> Self {
        Self::Message(TypeId::of::<T>())
    }
}
