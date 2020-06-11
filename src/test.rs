use crate::Activity;

struct TestActivity {
    inner: u32,
}

impl Activity for TestActivity {}

impl TestActivity {
    fn assert_eq_and_add(&mut self, eq: u32, add: u32) {
        assert_eq!(self.inner, eq);
        self.inner += add;
    }
}

#[test]
fn closure_registration() {
    let a = TestActivity { inner: 0 };
    let id = crate::activity(a);
    crate::register(id, |activity: &mut TestActivity| {
        activity.assert_eq_and_add(0, 1);
    });
    crate::update();
}

#[test]
#[should_panic]
fn closure_registration_negative() {
    let a = TestActivity { inner: 0 };
    let id = crate::activity(a);
    crate::register(id, |activity: &mut TestActivity| {
        activity.assert_eq_and_add(0, 1);
    });
    crate::update();
    crate::update();
}
