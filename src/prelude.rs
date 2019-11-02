pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;
pub(crate) use async_std::task;
pub(crate) use futures::{channel::mpsc::unbounded, Stream, StreamExt};
pub(crate) use std::{
    marker::Unpin,
    sync::{Arc, RwLock},
};
pub(crate) type Keywords = &'static [&'static str];
pub(crate) type Snapshot = Arc<RwLock<Arc<crate::Aggregator>>>;
