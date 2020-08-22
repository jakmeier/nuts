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
}

impl<ITEM> Default for ThreadLocalFifo<ITEM> {
    fn default() -> Self {
        ThreadLocalFifo {
            fifo: RefCell::new(VecDeque::new()),
        }
    }
}
