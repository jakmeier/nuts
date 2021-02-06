// @ START-DOC CRATE
//! Nuts is a library that offers a simple publish-subscribe API, featuring decoupled creation of the publisher and the subscriber.
//!
//! ## Quick first example
//! ```rust
//! struct Activity;
//! let activity = nuts::new_activity(Activity);
//! activity.subscribe(
//!     |_activity, n: &usize|
//!     println!("Subscriber received {}", n)
//! );
//! nuts::publish(17usize);
//! // "Subscriber received 17" is printed
//! nuts::publish(289usize);
//! // "Subscriber received 289" is printed
//! ```
//!
//! As you can see in the example above, no explicit channel between publisher and subscriber is necessary.
//! The call to `publish` is a static method that requires no state from the user.
//! The connection between them is implicit because both use `usize` as message type.
//!
//! Nuts enables this simple API by managing all necessary state in thread-local storage.
//! This is particularly useful when targeting the web. However, Nuts can be used on other platforms, too.
//! In fact, Nuts has no dependencies aside from std.
// @ END-DOC CRATE

// code quality
#![forbid(unsafe_code)]
#![deny(clippy::mem_forget)]
#![deny(clippy::print_stdout)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::unwrap_used)]
// docs
#![warn(missing_docs)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::missing_errors_doc)]
#![allow(clippy::needless_doctest_main)]

#[macro_use]
pub(crate) mod debug;

mod nut;

#[cfg(test)]
mod test;

pub use crate::nut::iac::managed_state::{DefaultDomain, DomainEnumeration, DomainState};
use core::any::Any;
pub use nut::activity::*;
pub use nut::iac::filter::*;

use nut::iac::managed_state::*;
use nut::iac::topic::*;

/// Consumes a struct and registers it as an Activity.
///
/// `nuts::new_activity(...)` is the simplest method to create a new activity.
/// It takes only a single argument, which can be any struct instance or primitive.
/// This object will be the private data for the activity.
///
/// An `ActivityId` is returned, which is a handle to the newly registered activity.
/// Use it to register callbacks on the activity.
///
/// ### Example:
// @ START-DOC NEW_ACTIVITY
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
/// // Activity moves into globally managed state, ID to handle it is returned
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
    let a = nut::new_activity(activity, DomainId::default(), LifecycleStatus::Active);
    #[cfg(feature = "verbose-debug-log")]
    debug_print!(
        "New activity {:?}({})",
        std::any::type_name::<A>(),
        a.id.index
    );
    a
}

/// Consumes a struct that is registered as an Activity that has access to the specified domain.
/// Use the returned `ActivityId` to register callbacks on the activity.
///
// @ START-DOC NEW_ACTIVITY_WITH_DOMAIN
/// ```rust
/// use nuts::{domain_enum, DomainEnumeration};
///
/// #[derive(Default)]
/// struct MyActivity;
/// struct MyMessage;
///
/// #[derive(Clone, Copy)]
/// enum MyDomain {
///     DomainA,
///     DomainB,
/// }
/// domain_enum!(MyDomain);
///
/// // Add data to domain
/// nuts::store_to_domain(&MyDomain::DomainA, 42usize);
///
/// // Register activity
/// let activity_id = nuts::new_domained_activity(MyActivity, &MyDomain::DomainA);
///
/// // Add event listener that listens to published `MyMessage` types and has also access to the domain data
/// activity_id.subscribe_domained(
///     |_my_activity, domain, msg: &MyMessage| {
///         // borrow data from the domain
///         let data = domain.try_get::<usize>();
///         assert_eq!(*data.unwrap(), 42);
///     }
/// );
///
/// // make sure the subscription closure is called
/// nuts::publish( MyMessage );
/// ```
// @ END-DOC NEW_ACTIVITY_WITH_DOMAIN
pub fn new_domained_activity<A, D>(activity: A, domain: &D) -> ActivityId<A>
where
    A: Activity,
    D: DomainEnumeration,
{
    let a = nut::new_activity(activity, DomainId::new(domain), LifecycleStatus::Active);
    #[cfg(feature = "verbose-debug-log")]
    debug_print!(
        "New activity {:?}({})",
        std::any::type_name::<A>(),
        a.id.index
    );

    a
}

/// Puts the data object to the domain, which can be accessed by all associated activities.
///
/// This function stores the data to the domain immediately if called outside of activities.
/// Inside activities, it will be delayed. However, any messages published after calling this function can
/// rely on the store to the domain to have completed when the corresponding subscribers are executed.
pub fn store_to_domain<D, T>(domain: &D, data: T)
where
    D: DomainEnumeration,
    T: core::any::Any,
{
    nut::write_domain(domain, data)
}

/// Send the message to all subscribed activities
///
// @ START-DOC PUBLISH
/// Any instance of a struct or primitive can be published, as long as its type is known at compile-time. (The same constraint as for Activities.)
/// Upon calling `nuts::publish`, all active subscriptions for the same type are executed and the published object will be shared with all of them.
///
/// ### Example
/// ```rust
/// struct ChangeUser { user_name: String }
/// pub fn main() {
///     let msg = ChangeUser { user_name: "Donald Duck".to_owned() };
///     nuts::publish(msg);
///     // Subscribers to messages of type `ChangeUser` will be notified
/// }
/// ```
// @ END-DOC PUBLISH
/// ### Advanced: Understanding the Execution Order
// @ START-DOC PUBLISH_ADVANCED
/// When calling `nuts::publish(...)`, the message may not always be published immediately. While executing a subscription handler from previous `publish`, all new messages are queued up until the previous one is completed.
/// ```rust
/// struct MyActivity;
/// let activity = nuts::new_activity(MyActivity);
/// activity.subscribe(
///     |_, msg: &usize| {
///         println!("Start of {}", msg);
///         if *msg < 3 {
///             nuts::publish( msg + 1 );
///         }
///         println!("End of {}", msg);
///     }
/// );
///
/// nuts::publish(0usize);
/// // Output:
/// // Start of 0
/// // End of 0
/// // Start of 1
/// // End of 1
/// // Start of 2
/// // End of 2
/// // Start of 3
/// // End of 3
/// ```
// @ END-DOC PUBLISH_ADVANCED
pub fn publish<A: Any>(a: A) {
    nut::publish_custom(a)
}

/// Returns a future of type `NutsResponse` which will resolve after the
/// message has been published and all subscribers have finished processing it.
pub async fn publish_awaiting_response<A: Any>(a: A) {
    nut::publish_custom_and_await(a).await;
}

/// Publish a message to a specific activity. The same as `id.private_message()` but works without an `ActivityId`.
///
/// The first type parameter must always be specified.
/// It determines the receiver of the message.
/// The message is ignored silently if no such activity has been registered or if it has no private channel for this message.
///
/// The second type parameter can usually be deferred by the compiler, it is the type of the message to be sent.
/// ### Example
// @ START-DOC PUBLISH_PRIVATE
/// ```rust
/// struct ExampleActivity {}
/// let id = nuts::new_activity(ExampleActivity {});
/// // `private_channel` works similar to `subscribe` but it owns the message.
/// id.private_channel(|_activity, msg: usize| {
///     assert_eq!(msg, 7);
/// });
/// // `send_to` must be used instead of `publish` when using private channels.
/// // Which activity receives the message is decide by the first type parameter.
/// nuts::send_to::<ExampleActivity, _>(7usize);
/// ```
// @ END-DOC PUBLISH_PRIVATE
pub fn send_to<RECEIVER: Any, MSG: Any>(msg: MSG) {
    nut::send_custom::<RECEIVER, MSG>(msg)
}

#[cfg(debug_assertions)]
/// Read some information about currently processing activities.
/// This should be called inside a panic hook.
///
/// This function is only available in debug mode as a runtime cost is associated with recording the necessary data at all times.
/// The correct flag for conditional compilation is `#[cfg(debug_assertions)]`.
///
/// # Example
/// ```
/// fn add_nuts_hook() {
/// #[cfg(debug_assertions)]
/// let previous_hook = std::panic::take_hook();
///     std::panic::set_hook(Box::new(move |panic_info| {
///         let nuts_info = nuts::panic_info();
///         #[cfg(features = "web-debug")]
///         web_sys::console::error_1(&nuts_info.into());
///         #[cfg(not(features = "web-debug"))]
///         eprintln!("{}", nuts_info);
///         previous_hook(panic_info)
///     }));
/// }
/// ```
pub fn panic_info() -> String {
    nut::nuts_panic_info()
        .unwrap_or_else(|| "NUTS panic hook: Failed to read panic info.".to_owned())
}
