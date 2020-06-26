use std::any::{Any, TypeId};

/// A topic for messages that can be published and subscribed to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Topic {
    // TODO: Shouldn't the name be like "BuiltinNotification" and "Messages" or something like that?
    Builtin(BuiltinTopic),
    Custom(CustomTopic),
}

/// Built-in topics are understood by webnut and it may generate events for it
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinTopic {
    /// On status change to active (not called if started as active)
    Enter,
    /// On status change to inactive
    Leave,
}

pub type CustomMessage = std::rc::Rc<dyn std::any::Any>;
pub type CustomTopic = TypeId;

impl Topic {
    pub(crate) fn enter() -> Self {
        Self::Builtin(BuiltinTopic::Enter)
    }
    pub(crate) fn leave() -> Self {
        Self::Builtin(BuiltinTopic::Leave)
    }
    pub(crate) fn custom<T: Any>() -> Self {
        Self::Custom(TypeId::of::<T>())
    }
}
