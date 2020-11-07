use std::{future::Future, task::Poll};

use crate::nut::Nut;

#[derive(Default)]
pub(crate) struct ResponseTracker {
    slots: Vec<SlotState>,
}

enum SlotState {
    Available,
    Occupied,
    Done,
}

pub(crate) struct Slot(usize);

#[allow(clippy::single_match)]
impl ResponseTracker {
    pub fn allocate(&mut self) -> Slot {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            match slot {
                SlotState::Available => {
                    *slot = SlotState::Occupied;
                    return Slot(i);
                }
                _ => {}
            }
        }
        let i = self.slots.len();
        self.slots.push(SlotState::Occupied);
        Slot(i)
    }
    pub fn done(&mut self, slot: &Slot) {
        self.slots[slot.0] = SlotState::Done;
    }
    fn free(&mut self, index: usize) {
        self.slots[index] = SlotState::Available;
    }
}

pub struct NutsResponse {
    index: usize,
}

impl NutsResponse {
    pub(crate) fn new(slot: &Slot) -> Self {
        Self { index: slot.0 }
    }
}

impl Future for NutsResponse {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Nut::with_response_tracker_mut(|response_tracker| {
            match response_tracker.slots[self.index] {
                SlotState::Available => panic!("Corrupted futures State"),
                SlotState::Occupied => Poll::Pending,
                SlotState::Done => {
                    response_tracker.free(self.index);
                    Poll::Ready(())
                }
            }
        })
    }
}
