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

#[test]
fn on_enter_and_leave_and_delete_for_inchoate_activity() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    let aid_slot: Rc<Cell<Option<ActivityId<TestActivity>>>> = Default::default();
    let aid_slot_clone = aid_slot.clone();

    main.subscribe(move |_, _: &Main| {
        let id = crate::new_domained_activity(a.clone(), &d);
        id.on_enter_domained(|activity: &mut TestActivity, domain| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });
        id.on_leave_domained(|activity: &mut TestActivity, domain| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(10);
        });
        id.on_delete_domained(|activity: TestActivity, domain| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(100);
        });
        id.set_status(LifecycleStatus::Inactive);
        id.set_status(LifecycleStatus::Active);
        aid_slot.set(Some(id));
    });

    assert_eq!(counter.get(), 0);
    crate::publish(Main);
    assert_eq!(counter.get(), 11);

    let aid = aid_slot_clone.get().unwrap();
    aid.set_status(LifecycleStatus::Inactive);
    assert_eq!(counter.get(), 21,);

    aid.set_status(LifecycleStatus::Active);
    assert_eq!(counter.get(), 22,);

    aid.set_status(LifecycleStatus::Deleted);
    assert_eq!(counter.get(), 132,);
}

#[test]
fn create_inchoate_domained_activity_and_subscribe_and_publish_privately() {
    let main = crate::new_activity(());
    let a = TestActivity::new();
    let b = (TestActivity::new(),);
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    let bid = crate::new_domained_activity(b, &d);
    bid.private_domained_channel(|_activity, domain, msg: &TestForInt| {
        let x: usize = *domain.get();
        assert_eq!(msg.0, x);
    });

    main.private_channel(move |_, _: Main| {
        let id = crate::new_domained_activity(a.clone(), &d);
        id.private_domained_channel(|activity: &mut TestActivity, domain, _: TestUpdateMsg| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });
        crate::send_to::<(TestActivity,), _>(TestForInt(7));
    });

    crate::send_to::<(), _>(Main);

    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::send_to::<TestActivity, _>(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::send_to::<TestActivity, _>(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}

#[test]
fn create_new_after_delete() {
    let main = crate::new_activity(());

    main.private_channel(move |_, _: Main| {
        let a = TestActivity::new();
        let id_a = crate::new_activity(a);
        id_a.set_status(LifecycleStatus::Deleted);
        let num_a: UncheckedActivityId = id_a.into();

        let b = (TestActivity::new(),);
        let id_b = crate::new_activity(b);
        let num_b: UncheckedActivityId = id_b.into();

        assert_ne!(num_a, num_b);
    });
    crate::send_to::<(), _>(Main);
}

#[test]
/// Create a (normal) activity A and register on_delete
/// Delete A, in A.on_delete:
///     Create (inchoate) activity B
///     Register on_delete in B
///     Delete B, in B.on_delete:
///         Create C and ensure nothing funky happened to the IDs
#[allow(non_snake_case)]
fn complex_scenario_0() {
    let A = TestActivity::new();
    let B = (TestActivity::new(), ());
    let C = (TestActivity::new(), (), ());

    let id_a = crate::new_activity(A);

    id_a.on_delete(move |_| {
        let id_b = crate::new_activity(B);
        let num_b: UncheckedActivityId = id_b.into();

        assert_ne!(num_b, id_a.into());

        id_b.on_delete(move |_| {
            let id_c = crate::new_activity(C);
            let num_c: UncheckedActivityId = id_c.into();
            assert_ne!(num_c, num_b);
            assert_ne!(num_c, id_a.into());
        });
        id_b.set_status(LifecycleStatus::Deleted);
    });

    id_a.set_status(LifecycleStatus::Deleted);
}
