/// A topic for messages that can be published and subscribed to
#[derive(Debug, Clone)]
pub enum Topic {
    Builtin(BuiltinTopic),
}

/// Built-in topics are understood by webnut and it may generate events for it
#[derive(Debug, Clone, Copy)]
pub enum BuiltinTopic {
    /// Messages are emitted for each frame displayed to the screen
    Draw,
    /// On status change to active (not called if started as active)
    Enter,
    /// On status change to inactive
    Leave,
    /// Emitted at a defined interval (or slower when overloaded)
    Update,
}

impl Topic {
    pub fn draw() -> Self {
        Self::Builtin(BuiltinTopic::Draw)
    }
    pub fn enter() -> Self {
        Self::Builtin(BuiltinTopic::Enter)
    }
    pub fn leave() -> Self {
        Self::Builtin(BuiltinTopic::Leave)
    }
    pub fn update() -> Self {
        Self::Builtin(BuiltinTopic::Update)
    }
}
