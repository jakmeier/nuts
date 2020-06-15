mod nut;
#[cfg(target_arch = "wasm32")]
mod web;

pub use crate::nut::iac::managed_state::DomainState;
pub use nut::activity::*;
pub use nut::iac::filter::*;
pub use nut::iac::topic::*;

use nut::iac::managed_state::*;
use nut::iac::publish::*;

#[cfg(target_arch = "wasm32")]
pub use web::*;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

/// A method on an activity. Can be registered dynamically on activities at runtime.
pub struct Method<ACTIVITY>(dyn Fn(&mut ACTIVITY, Option<&mut DomainState>));

/// Consumes a struct that is registered as an Activity.
/// Use the returned ActivityId to register callbacks on the activity.
///
/// start_active: Initial state of the activity
pub fn new_activity<A>(activity: A, start_active: bool) -> ActivityId<A>
where
    A: Activity,
{
    nut::new_activity(activity, DomainId::default(), start_active)
}

/// Consumes a struct that is registered as an Activity.
/// Use the returned ActivityId to register callbacks on the activity.
///
/// start_active: Initial state of the activity
pub fn new_domain_activity<A, D>(activity: A, domain: D, start_active: bool) -> ActivityId<A>
where
    A: Activity,
    D: DomainEnumeration
{
    nut::new_activity(activity, DomainId::new(domain), start_active)
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

/// Changes the active status of an activity.
/// If the status changes, the corresponding enter/leave subscriptions will be called.
pub fn set_active<A: Activity>(id: ActivityId<A>, active: bool) {
    nut::set_active(id, active)
}
