use core::cell::RefCell;
use std::collections::VecDeque;

/// FIFO queue that allows thread-local atomic pushing and popping.
/// No borrowing of internal data is possible, only moving data in and out.
/// No mutable access required for those operation.
///
/// Note that the chosen limitation prevents an implementation of Iterator for
/// this collection. `IntoIterator` would be possible but is mostly useless.
pub(crate) struct ThreadLocalFifo<ITEM> {
    fifo: RefCell<VecDeque<ITEM>>,
}

impl<ITEM> ThreadLocalFifo<ITEM> {
    pub(crate) fn push(&self, i: ITEM) {
        self.fifo.borrow_mut().push_back(i);
    }
    pub(crate) fn pop(&self) -> Option<ITEM> {
        self.fifo.borrow_mut().pop_front()
    }
    #[cfg(feature = "verbose-debug-log")]
    pub(crate) fn len(&self) -> usize {
        self.fifo.borrow().len()
    }
}

impl<ITEM: std::fmt::Debug> ThreadLocalFifo<ITEM> {
    #[cfg(feature = "verbose-debug-log")]
    pub(crate) fn events_debug_list(&self) -> String {
        let mut out = "(".to_owned();
        for e in self.fifo.borrow().iter() {
            out += &format!("{:?}, ", e);
        }
        if out.len() > 2 {
            out.remove(out.len() - 1);
            out.remove(out.len() - 1);
        }
        out += ")";
        out
    }
}
impl<ITEM> Default for ThreadLocalFifo<ITEM> {
    fn default() -> Self {
        ThreadLocalFifo {
            fifo: RefCell::new(VecDeque::new()),
        }
    }
}
