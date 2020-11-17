//! Encapsulation features that allow accessing the state stored inside the NUT but need no tracking inside the NUT.

use crate::debug::DebugTypeName;
use crate::nut::ExecError;
use crate::nut::Handler;
use crate::{Activity, ActivityId, DomainState, ManagedState, SubscriptionFilter};

/// Encapsulates an independent handler function that has full access to an activity (and domain).
pub struct Capsule {
    handler: Handler,
    type_name: DebugTypeName,
}

impl<A: Activity> ActivityId<A> {
    /// Create a new Capsule
    pub fn encapsulate<F>(&self, f: F) -> Capsule
    where
        F: Fn(&mut A) + 'static,
    {
        let handler =
            ManagedState::pack_closure_no_payload(f, *self, SubscriptionFilter::default());
        Capsule {
            handler,
            type_name: DebugTypeName::new::<A>(),
        }
    }
    /// Create a new Capsule that has access to the domain of the activity.
    pub fn encapsulate_domained<F>(&self, f: F) -> Capsule
    where
        F: Fn(&mut A, &mut DomainState) + 'static,
    {
        let handler =
            ManagedState::pack_closure_domained_no_payload(f, *self, SubscriptionFilter::default());
        Capsule {
            handler,
            type_name: DebugTypeName::new::<A>(),
        }
    }
}

impl Capsule {
    /// Borrow NUT state mutably and execute the encapsulated handler.
    ///
    /// This can only work if NUTS is not executing already.
    /// Usually, this should only be used as a callback event handler, which will be executed when the main thread is not active.
    ///
    /// If you really must, you can also use it (completely safely) in the main thread, as long as it is outside of any subscriptions/callbacks.
    ///
    /// # Errors
    /// If nuts is already active, this will return en `ExecError::NutsAlreadyActive`
    pub fn execute(&self) -> Result<(), ExecError> {
        crate::nut::try_exec_single_handler(&self.handler, &self.type_name)
    }
}
