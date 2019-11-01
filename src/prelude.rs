pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;
pub(crate) type Receiver<T> = futures::channel::mpsc::UnboundedReceiver<T>;
pub(crate) use std::marker::Unpin;
pub(crate) use futures::{
	Stream, StreamExt,
	compat::{Future01CompatExt},
	channel::mpsc::unbounded,
};
pub(crate) use async_std::task;
