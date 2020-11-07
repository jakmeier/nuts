mod activity_container;
mod lifecycle;

pub(crate) use activity_container::*;
pub use lifecycle::*;

use crate::nut::iac::{filter::SubscriptionFilter, managed_state::DomainId};
use crate::*;
use core::any::Any;
use std::ops::{Index, IndexMut};

// @ START-DOC ACTIVITY
/// Activities are at the core of Nuts.
/// From the globally managed data, they represent the active part, i.e. they can have event listeners.
/// The passive counter-part is defined by `DomainState`.
///
/// Every struct that has a type with static lifetime (anything that has no lifetime parameter that is determined only at runtime) can be used as an Activity.
/// You don't have to implement the `Activity` trait yourself, it will always be automatically derived if possible.
///
/// To create an activity, simply register the object that should be used as activity, using `nuts::new_activity` or one of its variants.
// @ END-DOC ACTIVITY
pub trait Activity: Any {}
impl<T: Any> Activity for T {}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
/// Handle to an `Activity` that has been registered, with a type parameter to track the activity's type.
/// Can be used to add type-checked closures to the activity, which will be used as event listeners.
///
/// Implements `Copy` and `Clone`
pub struct ActivityId<A> {
    pub(crate) id: UncheckedActivityId,
    pub(crate) domain_index: DomainId,
    phantom: std::marker::PhantomData<A>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
/// Pointer to an activity that has been registered.
/// Can be used to set the lifecycle stats of activities.
///
/// The information about the activity's type is lost at this point.
/// Therefore, this id cannot be used to register closures.
pub struct UncheckedActivityId {
    pub(crate) index: usize,
}

impl<A: Activity> ActivityId<A> {
    pub(crate) fn new(index: usize, domain_index: DomainId) -> Self {
        Self {
            id: UncheckedActivityId { index },
            domain_index,
            phantom: Default::default(),
        }
    }
    /// Registers a callback closure that is called when an activity changes from inactive to active.
    pub fn on_enter<F>(&self, f: F)
    where
        F: Fn(&mut A) + 'static,
    {
        crate::nut::register_no_payload(*self, f, Topic::enter(), SubscriptionFilter::no_filter())
    }
    /// Same as `on_enter` but with domain access in closure
    pub fn on_enter_domained<F>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState) + 'static,
    {
        crate::nut::register_domained_no_payload(
            *self,
            f,
            Topic::enter(),
            SubscriptionFilter::no_filter(),
        )
    }
    /// Registers a callback closure that is called when an activity changes from active to inactive.
    pub fn on_leave<F>(&self, f: F)
    where
        F: Fn(&mut A) + 'static,
    {
        crate::nut::register_no_payload(*self, f, Topic::leave(), SubscriptionFilter::no_filter())
    }
    /// Same as `on_leave` but with domain access in closure
    pub fn on_leave_domained<F>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState) + 'static,
    {
        crate::nut::register_domained_no_payload(
            *self,
            f,
            Topic::leave(),
            SubscriptionFilter::no_filter(),
        )
    }
    /// Registers a callback closure that is called when an activity is deleted.
    pub fn on_delete<F>(&self, f: F)
    where
        F: FnOnce(A) + 'static,
    {
        crate::nut::register_on_delete(*self, f);
    }
    /// Same as `on_delete` but with domain access in closure
    pub fn on_delete_domained<F>(&self, f: F)
    where
        F: FnOnce(A, &mut DomainState) + 'static,
    {
        crate::nut::register_domained_on_delete(*self, f);
    }

    /// Registers a callback closure on an activity with a specific topic to listen to.
    ///
    /// By default, the activity will only receive calls when it is active.
    /// Use `subscribe_masked` for more control over this behavior.
    ///
    /// ### Example
    // @ START-DOC SUBSCRIBE_EXAMPLE
    /// ```rust
    /// struct MyActivity { id: usize };
    /// struct MyMessage { text: String };
    ///
    /// pub fn main() {
    ///     let activity = nuts::new_activity(MyActivity { id: 0 } );
    ///     activity.subscribe(
    ///         |activity: &mut MyActivity, message: &MyMessage|
    ///         println!("Subscriber with ID {} received text: {}", activity.id, message.text)
    ///     );
    /// }
    /// ```
    /// In the example above, a subscription is created that waits for messages of type `MyMessage` to be published.
    /// So far, the code inside the closure is not executed and nothing is printed to the console.
    ///
    /// Note that the first argument of the closure is a mutable reference to the activity object.
    /// The second argument is a read-only reference to the published message.
    /// Both types must match exactly or otherwise the closure will not be accepted by the compiler.
    ///
    /// A function with the correct argument types can also be used to subscribe.
    /// ```rust
    /// struct MyActivity { id: usize };
    /// struct MyMessage { text: String };
    ///
    /// pub fn main() {
    ///     let activity = nuts::new_activity(MyActivity { id: 0 } );
    ///     activity.subscribe(MyActivity::print_text);
    /// }
    ///
    /// impl MyActivity {
    ///     fn print_text(&mut self, message: &MyMessage) {
    ///         println!("Subscriber with ID {} received text: {}", self.id, message.text)
    ///     }
    /// }
    /// ```
    // @ END-DOC SUBSCRIBE_EXAMPLE
    pub fn subscribe<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register(*self, f, Default::default())
    }
    /// Same as [subscribe](#method.subscribe) but gives mutable access to the message object.
    ///
    /// Make sure to use the correct signature for the function, the Rust compiler may give strange error messages otherwise.
    /// For example, the message must be borrowed by the subscription handler.
    pub fn subscribe_mut<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &mut MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_mut(*self, f, Default::default())
    }
    /// Registers a callback closure on an activity with a specific topic to listen to.
    /// This variant takes ownership of the message.
    /// Only subscription per type is allowed. Othwerise, a pnic will occur when publishing.
    ///
    /// When using this variant, the subscription handler should take ownership of the message.
    /// It will compile if it is borrowed instead, but then it will also expect a reference to be published. (Which usually doesn't work due to lifetimes)
    /// Then, it will not react to normal published messages and can be difficult to debug.
    pub fn subscribe_owned<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_owned(*self, f, Default::default())
    }

    /// Registers a callback closure on an activity with a specific topic to listen to.
    /// Has mutable access to the `DomainState` object.
    ///
    /// By default, the activity will only receive calls when it is active.
    /// Use `subscribe_domained_masked` for more control over this behavior.
    ///
    /// Make sure to use the correct signature for the function, the Rust compiler may give strange error messages otherwise.
    /// For example, the message must be borrowed by the subscription handler.
    ///
    /// # Panics
    /// Panics if the activity has not been registered with a domain.    
    pub fn subscribe_domained<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState, &MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained(*self, f, Default::default())
    }
    /// Same as [`subscribe_domained`](#method.subscribe_domained) but gives mutable access to the message object.
    pub fn subscribe_domained_mut<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState, &mut MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained_mut(*self, f, Default::default())
    }
    /// Registers a callback closure on an activity with a specific topic to listen to and access to the domain.
    /// This variant takes ownership of the message.
    /// Only subscription per type is allowed. Otherwise, a panic will occur when publishing.
    pub fn subscribe_domained_owned<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained_owned(*self, f, Default::default())
    }

    /// Registers a callback closure on an activity with a specific topic to listen to with filtering options.
    pub fn subscribe_masked<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, &MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register(*self, f, mask)
    }
    /// Same as [`subscribe_masked`](#method.subscribe_masked) but gives mutable access to the message object.
    pub fn subscribe_masked_mut<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, &mut MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_mut(*self, f, mask)
    }

    /// Registers a callback closure on an activity with a specific topic to listen to with filtering options.
    /// Has mutable access to the `DomainState` object.
    ///
    /// # Panics
    /// Panics if the activity has not been registered with a domain.
    pub fn subscribe_domained_masked<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, &mut DomainState, &MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained(*self, f, mask)
    }
    /// Same as [`subscribe_domained_masked`](#method.subscribe_domained_masked) but gives mutable access to the message object.
    pub fn subscribe_domained_masked_mut<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, &mut DomainState, &mut MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained_mut(*self, f, mask)
    }

    /// Changes the lifecycle status of the activity
    ///
    /// # Panics
    /// If status is set to something other than Deleted after it has been Deleted
    pub fn set_status(&self, status: LifecycleStatus) {
        crate::nut::set_status((*self).into(), status);
    }
}

impl UncheckedActivityId {
    /// Changes the lifecycle status of the activity
    ///
    /// # Panics
    /// If status is set to something other than Deleted after it has been Deleted
    pub fn set_status(&self, status: LifecycleStatus) {
        crate::nut::set_status(*self, status);
    }
}

impl<A> Copy for ActivityId<A> {}
impl<A> Clone for ActivityId<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Into<UncheckedActivityId> for ActivityId<A> {
    fn into(self) -> UncheckedActivityId {
        self.id
    }
}
