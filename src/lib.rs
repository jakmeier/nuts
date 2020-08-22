// @ START-DOC CRATE
//! Nuts is a library that offers a simple publish-subscribe API.
//!
//! ## Quick first example
//! ```rust
//! struct Activity;
//! let activity = nuts::new_activity(Activity);
//! activity.subscribe(|_activity, n: &usize| println!("Subscriber received {}", n) );
//! nuts::publish(17usize);
//! // "Subscriber received 17" is printed
//! ```
//!
//! As you can see in the example above, no explicit channel between publisher and subscriber is necessary.
//! They are only connected because both of them used `usize` as message type.
//!
//! Nuts enables this simple API by managing all necessary state in thread-local storage.
//! This is particularly useful when targeting the web.
//! However, Nuts has no dependencies (aside from std) and therefore can be used on other platforms, too.
// @ END-DOC CRATE

// code quality
#![forbid(unsafe_code)]
#![deny(clippy::mem_forget)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::needless_pass_by_value)]
// docs
#![warn(missing_docs)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::missing_errors_doc)]

mod nut;

pub use crate::nut::iac::managed_state::{DomainEnumeration, DomainState};
use core::any::Any;
pub use nut::activity::*;
pub use nut::iac::filter::*;

use nut::iac::managed_state::*;
use nut::iac::topic::*;

/// A method on an activity. Can be registered dynamically on activities at runtime.
pub struct Method<ACTIVITY>(dyn Fn(&mut ACTIVITY, Option<&mut DomainState>));

/// Consumes a struct and registers it as an Activity.
///
// @ START-DOC NEW_ACTIVITY
/// `nuts::new_activity(...)` is the simplest method to create a new activity.
/// It takes only a single argument, which can be any struct instance or primitive.
/// This object will be the private data for the activity.
///
/// An `ActivityId` is returned, which is a handle to the newly registered activity.
/// Use it to register callbacks on the activity.
///
/// # Example:
/// ```rust
/// #[derive(Default)]
/// struct MyActivity {
///     round: usize
/// }
/// struct MyMessage {
///     no: usize
/// }
///
/// // Create activity
/// let activity = MyActivity::default();
/// // Activity moves into globally managed stated, ID to handle it is returned
/// let activity_id = nuts::new_activity(activity);
///
/// // Add event listener that listens to published `MyMessage` types
/// activity_id.subscribe(
///     |my_activity, msg: &MyMessage| {
///         println!("Round: {}, Message No: {}", my_activity.round, msg.no);
///         my_activity.round += 1;
///     }
/// );
///
/// // prints "Round: 0, Message No: 1"
/// nuts::publish( MyMessage { no: 1 } );
/// // prints "Round: 1, Message No: 2"
/// nuts::publish( MyMessage { no: 2 } );
/// ```
// @ END-DOC NEW_ACTIVITY
pub fn new_activity<A>(activity: A) -> ActivityId<A>
where
    A: Activity,
{
    nut::new_activity(activity, DomainId::default(), LifecycleStatus::Active)
}

/// Consumes a struct that is registered as an Activity with access to the specified domain.
/// Use the returned `ActivityId` to register callbacks on the activity.
pub fn new_domained_activity<A, D>(activity: A, domain: &D) -> ActivityId<A>
where
    A: Activity,
    D: DomainEnumeration,
{
    nut::new_activity(activity, DomainId::new(domain), LifecycleStatus::Active)
}

/// Puts the data object to the domain, which can be accessed by all associated activities.
///
/// This function is only valid outside of activities.
/// Inside activities, only access domains through the handlers borrowed access.
/// Typically, this functino is only used for initialization of the domain state.
pub fn store_to_domain<D, T>(domain: &D, data: T)
where
    D: DomainEnumeration,
    T: core::any::Any,
{
    nut::write_domain(domain, data).expect("You cannot use `store_to_domain` after initialization.")
}

/// Send the message to all subscribed activities
pub fn publish<A: Any>(a: A) {
    nut::publish_custom(a)
}
