//! Test suite for domain usage.
use super::*;

#[test]
fn store_to_domain_inside_activity() {
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);
    let id = crate::new_domained_activity(a, &d);
    id.subscribe_domained(|_activity, domain, msg: &TestForInt| {
        let x: usize = *domain.get();
        assert_eq!(msg.0, x);
    });
    id.subscribe_domained(|_activity, domain, _msg: &TestUpdateMsg| {
        let d = TestDomains::DomainA;
        // check we can read domain values
        let x: usize = *domain.get();
        assert_eq!(7usize, x);
        // check we can write domain values
        *domain.get_mut() = 8usize;
        let x: usize = *domain.get();
        assert_eq!(8usize, x);
        // Check we can store to arbitrary domain (which will have to be deferred in this example)
        crate::store_to_domain(&d, 9usize);

        // Update should no be visible, yet, as we are locking the domain
        let x: usize = *domain.get();
        assert_eq!(8, x);
    });
    // Check domain value before
    crate::publish(TestForInt(7));
    // Update value from inside the subscriber
    crate::publish(TestUpdateMsg);
    // Check update has been completed
    crate::publish(TestForInt(9));
}
