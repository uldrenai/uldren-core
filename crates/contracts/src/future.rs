//! Boxed future alias that keeps asynchronous trait methods object-safe.

use std::future::Future;
use std::pin::Pin;

/// A heap-allocated `Send` future, used so asynchronous provider methods stay
/// object-safe and can be held behind `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
