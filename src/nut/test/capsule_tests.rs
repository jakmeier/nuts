use super::*;
#[test]
fn encapsulate_and_call_in_top_level() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
    let capsule = id.encapsulate(|activity: &mut TestActivity| {
        activity.inc(1);
    });

    assert_eq!(counter.get(), 0, "Closure called before calling capsule");
    capsule.execute().expect("Failed executing capsule");
    assert_eq!(counter.get(), 1);
    capsule.execute().expect("Failed executing capsule");
    assert_eq!(counter.get(), 2);
}

#[test]
fn with_domain() {
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);
    let counter = a.shared_counter_ref();
    let id = crate::new_domained_activity(a, &d);
    let capsule =
        id.encapsulate_domained(|activity: &mut TestActivity, domain: &mut DomainState| {
            let x: usize = *domain.get();
            assert_eq!(7, x);
            activity.inc(1);
        });

    assert_eq!(counter.get(), 0, "Closure called before calling capsule");
    capsule.execute().expect("Failed executing capsule");
    assert_eq!(counter.get(), 1);
    capsule.execute().expect("Failed executing capsule");
    assert_eq!(counter.get(), 2);
}
