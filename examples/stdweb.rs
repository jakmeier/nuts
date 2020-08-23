//! Run with `cargo web start --example stdweb --target wasm32-unknown-unknown`

use core::any::Any;
use nuts::{LifecycleStatus, SubscriptionFilter};
use stdweb::traits::*;
use stdweb::web::event::ClickEvent;
use stdweb::web::*;

/* Activities */
/// Controls the text display at the top.
struct DisplayActivity {
    text: Element,
}

/* Messages */
/// A number to add to a counter stored in the default domain
#[derive(Copy, Clone)]
struct AddToSum(usize);
/// A number to add to a counter stored in the default domain
#[derive(Copy, Clone)]
struct SubtractFromSum(usize);
/// A number to add to a counter stored in the default domain
#[derive(Copy, Clone)]
struct DivideSum(usize);
/// Tells all activities that the sum has changed
struct SumUpdated(usize);

/* Domain Data */
/// Number to be added to the global counter
struct Sum(usize);

/// Helper struct that controls an HTML element and publishes a static event when clicked.
struct ButtonActivity {
    div: Element,
}

pub fn main() {
    // Add styles to the body
    stdweb::web::document()
        .body()
        .expect("body not found")
        .set_attribute("style", "text-align: center")
        .expect("browser error");

    // Store a `Sum` value of 0 to the default domain.
    // The domain is like a small database shared among activities.
    // In a real life scenario, typically more complex data is shared in domains instead of simple numbers.
    // The default domain is used because all activities with a domain require to share the same state.
    nuts::store_to_domain(&nuts::DefaultDomain, Sum(0));

    // Create an activity that listens to `SumUpdated` messages and displays a text in a div
    let activity = nuts::new_activity(DisplayActivity::new());
    activity.subscribe(DisplayActivity::update_text);

    // Add a button that adds 1 to the sum
    let adder = ButtonActivity::new(AddToSum(1), &format!("plus {}", 1));
    // Domain access is required in the subscription handler in order to read and update the sum.
    // Therefore, `subscribe_domained` is used.
    adder.subscribe_domained(|_activity, domain_state, msg: &AddToSum| {
        // `sum` is borrowed mutably from the domain state
        let sum: &mut Sum = domain_state
            .try_get_mut()
            .expect("Sum not stored in domain");
        sum.0 += msg.0;
        // Send update notification to allow other activities to perform updates
        nuts::publish(SumUpdated(sum.0));
    });

    // Add a button that subtracts 1 from the sum. (Analogue to add button)
    let subtracter = ButtonActivity::new(SubtractFromSum(1), &format!("minus {}", 1));
    subtracter.subscribe_domained(|_activity, domain_state, msg: &SubtractFromSum| {
        let sum: &mut Sum = domain_state
            .try_get_mut()
            .expect("Sum not stored in domain");
        // Careful! usize underflow is possible here. Usually, I would perform a check here.
        // But for the sake of this example, we will instead ensure that this code never gets called when the sum is smaller than 1
        sum.0 -= msg.0;
        nuts::publish(SumUpdated(sum.0));
    });

    // change the subtracting activity to inactive to avoid an underflow (sum starts at 0)
    // The subscription for subtracting will thus not be executed when the button is clicked.
    subtracter.set_status(LifecycleStatus::Inactive);

    // After each sum update, enable the activity only when no underflow would occur.
    // The `subscribe_masked` variant is used for the subscription to receive events even in inactive state.
    subtracter.subscribe_masked(
        SubscriptionFilter::no_filter(),
        // `subtracter` is moved into the closure.
        // This works without lifetime errors because ActivityId implements `Copy`.
        move |_activity, sum: &SumUpdated| {
            if sum.0 > 0 {
                // changes the subtracting activity to active
                subtracter.set_status(LifecycleStatus::Active);
            } else {
                subtracter.set_status(LifecycleStatus::Inactive);
            }
        },
    );

    // Add a button that divides by 3 but is only active if the sum is a multiple of 3
    let divider = ButtonActivity::new(DivideSum(3), &format!("divide by {}", 3));
    divider.subscribe_domained(|_activity, domain_state, msg: &DivideSum| {
        let sum: &mut Sum = domain_state
            .try_get_mut()
            .expect("Sum not stored in domain");
        sum.0 /= msg.0;
        nuts::publish(SumUpdated(sum.0));
    });
    divider.subscribe_masked(
        SubscriptionFilter::no_filter(),
        move |_activity, sum: &SumUpdated| {
            if sum.0 % 3 == 0 {
                divider.set_status(LifecycleStatus::Active);
            } else {
                divider.set_status(LifecycleStatus::Inactive);
            }
        },
    );
}

impl DisplayActivity {
    fn new() -> Self {
        let background = stdweb::web::document().create_element("div").unwrap();
        stdweb::web::document()
            .body()
            .expect("body not found")
            .append_child(&background);

        let text = stdweb::web::document().create_element("p").unwrap();
        text.set_attribute("style", "font-size: xxx-large; font-weight: bold;").expect("browser error");
        background.append_child(&text);

        let mut out = Self { text };
        out.update_text(&SumUpdated(0));
        out
    }
    fn update_text(&mut self, sum: &SumUpdated) {
        self.text.set_text_content(&format!("Sum = {}", sum.0));
    }
}

impl ButtonActivity {
    /// Creates a new button and registers a new activity to Nuts to control it.
    /// The button will change its style based on the lifecycle status of the activity.
    fn new(msg: impl Any + Copy, text: &str) -> nuts::ActivityId<Self> {
        let div = stdweb::web::document().create_element("div").unwrap();
        stdweb::web::document()
            .body()
            .expect("body not found")
            .append_child(&div);
        div.set_attribute(
            "style",
            "background-color: orchid; padding: 50px; margin: 5px;",
        )
        .expect("browser error");
        div.set_text_content(text);
        div.add_event_listener(move |_e: ClickEvent| {
            nuts::publish(msg);
        });
        let activity = nuts::new_domained_activity(Self { div }, &nuts::DefaultDomain);
        // Change button HTML style when activity is not active
        activity.on_leave(|button| {
            button
                .div
                .set_attribute(
                    "style",
                    "background-color: grey; padding: 50px; margin: 5px; font-style: italic;",
                )
                .expect("browser error")
        });
        activity.on_enter(|button| {
            button
                .div
                .set_attribute(
                    "style",
                    "background-color: orchid; padding: 50px; margin: 5px;",
                )
                .expect("browser error")
        });
        activity
    }
}
