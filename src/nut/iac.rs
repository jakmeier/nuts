//! Inter Activity Communication
//! 
//! This module contains all the glue necessary for activities to work together.
//! 
//! A publish-subscribe model is used for scheduling, notifications, and low-bandwidth message bandwidth.
//! 
//! TODO: model for shared memory is planned for higher bandwidth communication.

pub(crate) mod filter;
pub(crate) mod publish;
pub(crate) mod topic;