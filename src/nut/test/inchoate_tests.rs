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

#[test]
fn create_inchoate_domained_activity_and_subscribe() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    main.subscribe(move |_, _: &Main| {
        let id = crate::new_domained_activity(a.clone(), &d);
        id.subscribe_domained(|activity: &mut TestActivity, domain, _: &TestUpdateMsg| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
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

#[test]
fn create_inchoate_domained_activity_and_subscribe_and_publish() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let b = (TestActivity::new(),);
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    let bid = crate::new_domained_activity(b, &d);
    bid.subscribe_domained(|_activity, domain, msg: &TestForInt| {
        let x: usize = *domain.get();
        assert_eq!(msg.0, x);
    });

    main.subscribe(move |_, _: &Main| {
        let id = crate::new_domained_activity(a.clone(), &d);
        id.subscribe_domained(|activity: &mut TestActivity, domain, _: &TestUpdateMsg| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });
        crate::publish(TestForInt(7));
    });

    crate::publish(Main);

    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}

#[test]
fn create_inchoate_domained_activity_and_subscribe_and_publish_to_inchoate_activity() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    main.subscribe(move |_, _: &Main| {
        let id = crate::new_domained_activity(a.clone(), &d);
        id.subscribe_domained(|_activity, domain, msg: &TestForInt| {
            let x: usize = *domain.get();
            assert_eq!(msg.0, x);
        });
        id.subscribe_domained(|activity: &mut TestActivity, domain, _: &TestUpdateMsg| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });
        crate::publish(TestForInt(7));
    });

    crate::publish(Main);

    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}

#[test]
fn queue_message_and_add_inchoate_subscriber() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    main.subscribe(move |_, _: &Main| {
        crate::publish(TestForInt(7));
        let id = crate::new_activity(a.clone());
        id.subscribe_domained(|_activity, domain, msg: &TestForInt| {
            let x: usize = *domain.get();
            assert_eq!(msg.0, x);
        });
        id.subscribe_domained(|activity: &mut TestActivity, domain, _: &TestUpdateMsg| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });
    });

    crate::publish(Main);
}

#[test]
// Simple test to make sure nothing panics. More detailed tests afterwards.
fn delete_inchoate_activity() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7u32);

    main.subscribe(move |_, _: &Main| {
        let id = crate::new_activity(a.clone());
        id.set_status(LifecycleStatus::Deleted);
    });
    crate::publish(Main);
}

#[test]
fn inchoate_ondelete() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7u32);

    main.subscribe(move |_, _: &Main| {
        crate::publish(TestForInt(7));
        let id = crate::new_activity(a.clone());
        id.on_delete(|a| a.inc(10));
        id.set_status(LifecycleStatus::Deleted);
    });
    assert_eq!(counter.get(), 0);
    crate::publish(Main);
    assert_eq!(counter.get(), 10);
}

#[test]
fn delete_inchoate_activity_with_subscriber_and_on_delete() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7u32);

    main.subscribe(move |_, _: &Main| {
        crate::publish(TestForInt(7));
        let id = crate::new_domained_activity(a.clone(), &d);
        id.subscribe_domained(|_activity, _domain, _msg: &TestMessage| {
            panic!("Activity should be deleted by now, why is the subscriber called?")
        });
        id.on_delete_domained(|a, domain| {
            let number: u32 = *domain.get();
            a.inc(number + 5);
        });
        id.set_status(LifecycleStatus::Deleted);
    });

    assert_eq!(counter.get(), 0);
    crate::publish(Main);
    assert_eq!(counter.get(), 12);
    crate::publish(TestMessage(0));
}
