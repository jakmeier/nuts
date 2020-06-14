mod activity;
mod filter;
mod nut;
mod publish;
mod topic;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(test)]
mod test;

pub use activity::*;
pub use filter::*;
pub use topic::*;

#[cfg(target_arch = "wasm32")]
pub use web::*;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

use publish::*;

use crate::Activity;

/// Consumes a struct that is registered as an Activity.
/// Use the returned ActivityId to register callbacks on the activity.
///
/// start_active: Initial state of the activity
pub fn new_activity<A>(activity: A, start_active: bool) -> ActivityId<A>
where
    A: Activity,
{
    nut::new_activity(activity, start_active)
}

/// Explicitly call all update methods on all activities
/// (Usually not necessary when using `auto_update`)
pub fn update() {
    nut::publish_builtin(GlobalNotification::Update)
}

/// Explicitly call all draw methods on all activities
/// (Usually not necessary when using `auto_draw`)
pub fn draw() {
    nut::publish_builtin(GlobalNotification::Draw)
}

/// Registers a callback closure on an activity with a specific topic to listen to.
///
/// By default, the activity will only receive calls when it is active.
/// Use `subscribe_masked` for more control over this behavior.
pub fn subscribe<A, F>(id: ActivityId<A>, topic: Topic, f: F)
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    nut::register(id, topic, f, Default::default())
}

/// Registers a callback closure on an activity with a specific topic to listen to.
pub fn subscribe_masked<A, F>(id: ActivityId<A>, topic: Topic, mask: SubscriptionFilter, f: F)
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    nut::register(id, topic, f, mask)
}

/// Changes the active status of an activity.
/// If the status changes, the corresponding enter/leave subscriptions will be called.
pub fn set_active<A: Activity>(id: ActivityId<A>, active: bool) {
    nut::set_active(id, active)
}
