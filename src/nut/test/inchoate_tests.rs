//! Testing creation and managing of activities while a broadcast is inflight.
use super::*;

struct Main;

#[test]
fn create_inchoate_activity() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let aid_slot: Rc<Cell<Option<ActivityId<TestActivity>>>> = Default::default();
    let aid_slot_clone = aid_slot.clone();
    main.subscribe(move |_, _: &Main| {
        let aid = crate::new_activity(a.clone());
        aid_slot.set(Some(aid));
    });

    crate::publish(Main);

    let id = aid_slot_clone.get().unwrap();
    id.subscribe(|activity: &mut TestActivity, _: &TestUpdateMsg| {
        activity.inc(1);
    });
    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}

#[test]
fn create_inchoate_activity_and_subscribe() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    main.subscribe(move |_, _: &Main| {
        let id = crate::new_activity(a.clone());
        id.subscribe(|activity: &mut TestActivity, _: &TestUpdateMsg| {
            activity.inc(1);
        });
    });

    crate::publish(Main);

    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}
