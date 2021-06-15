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
///
/// It is important to understand that Activities are uniquely defined by their type.
/// You cannot create two activities from the same type. (But you can, for example, create a wrapper type around it.)
/// This allows activities to be referenced by their type, which must be known at run-time.
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

/// This type is used for subscriptions without activity. It is zero sized, hence should be a zero-cost abstraction.
pub(crate) struct NotAnActivity;

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
    /// Multiple handlers can be registered.
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
    /// Multiple handlers can be registered.
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
    /// Only one handler can be registered because it takes ownership of the data.
    /// A second registration will overwrite the first handler.
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

    /// Registers a callback closure on an activity with a specific topic to listen to.
    /// Messages sent with `nuts::publish()` are NOT received, only messages sent with `nuts::send_to()`.
    ///
    /// Attention! The handler takes ownership of the message.
    /// It will compile if it is borrowed instead, but then it will also expect a reference to be published. (Which usually doesn't work due to lifetimes)
    /// Then, it will not react to normally sent messages and can be difficult to debug.
    ///
    /// Since the listener takes ownership, it is not possible to have more than one private channel active for the same activity at the same time.
    /// If multiple private channels are added to an activity, only the last listener is retained. (Older ones are replaced and deleted)
    pub fn private_channel<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_owned(*self, f, Default::default())
    }

    /// Variant of `private_channel` with access to the domain state.
    ///
    /// # Panics
    /// Panics if the activity has not been registered with a domain.   
    pub fn private_domained_channel<F, MSG>(&self, f: F)
    where
        F: Fn(&mut A, &mut DomainState, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained_owned(*self, f, Default::default())
    }

    /// Variant of `private_channel` with subscription mask.
    pub fn private_channel_masked<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_owned(*self, f, mask)
    }

    /// Variant of `private_channel` with access to the domain state and subscription mask.
    ///
    /// # Panics
    /// Panics if the activity has not been registered with a domain.   
    pub fn private_domained_channel_masked<F, MSG>(&self, mask: SubscriptionFilter, f: F)
    where
        F: Fn(&mut A, &mut DomainState, MSG) + 'static,
        MSG: Any,
    {
        crate::nut::register_domained_owned(*self, f, mask)
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

    /// Publish a message to a specific activity.
    ///
    /// If you lack access to an `ActivityId`, use `nuts::send_to()` or `UncheckedActivityId::private_message`.
    /// Both are equivalent.
    pub fn private_message<MSG: Any>(&self, msg: MSG) {
        let id: UncheckedActivityId = (*self).into();
        id.private_message(msg);
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
    /// Publish a message to a specific activity.
    ///
    /// If you lack access to an `UncheckedActivityId`, use `nuts::send_to()`, it is equivalent.
    pub fn private_message<A: Any>(&self, msg: A) {
        nut::send_custom_by_id(msg, *self)
    }
    /// A unique number for the activity.
    /// Can be used for serialization in combination with `forge_from_usize`
    pub fn as_usize(&self) -> usize {
        self.index
    }
    /// Can be used for deserialization in combination with `as_usize`
    ///
    /// If used in any other way, you might experience panics.
    /// Right now, there should still be no UB but that might change in future versions.
    pub fn forge_from_usize(index: usize) -> Self {
        Self { index }
    }
}

impl NotAnActivity {
    pub fn id() -> ActivityId<NotAnActivity> {
        ActivityId::<NotAnActivity>::new(0, DomainId::default())
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
