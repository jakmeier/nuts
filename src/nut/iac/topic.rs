use core::any::{Any, TypeId};

/// A topic for messages that can be published and subscribed to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Topic {
    /// Topic for a builtin event
    BuiltinEvent(BuiltinEvent),
    /// Topic for a message type, where type is a Rust type (core::any::TypeId). Many receivers can coexists for each published message.
    PublicMessage(TypeId),
    /// Topic for a message type, where type is a Rust type (core::any::TypeId). Only one receiver can exist per activity and each message must be sent to exactly one activity.
    PrivateMessage(TypeId),
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
    pub(crate) fn public_message<T: Any>() -> Self {
        Self::PublicMessage(TypeId::of::<T>())
    }
    pub(crate) fn private_message<T: Any>() -> Self {
        Self::PrivateMessage(TypeId::of::<T>())
    }
    pub(crate) fn unqiue_per_activity(&self) -> bool {
        match self {
            Self::BuiltinEvent(_) | Self::PublicMessage(_) => false,
            Self::PrivateMessage(_) => true,
        }
    }
}
