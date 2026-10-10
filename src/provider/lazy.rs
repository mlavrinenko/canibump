//! A provider that starts when first asked.

use std::sync::{Arc, OnceLock};

use super::CheckError;

/// A provider started, or why it could not start.
pub(super) type Started<T> = Result<Arc<T>, CheckError>;

/// A provider built by `start` on the first [`Lazy::get`], and never again.
///
/// The outcome is kept, a failure too: a provider that cannot start is not
/// retried, and every ask hears the same reason.
pub(super) struct Lazy<T: ?Sized> {
    start: Box<dyn Fn() -> Started<T> + Send + Sync>,
    started: OnceLock<Started<T>>,
}

impl<T: ?Sized> Lazy<T> {
    /// A provider that `start` builds when it is first asked for.
    pub(super) fn new(start: impl Fn() -> Started<T> + Send + Sync + 'static) -> Self {
        Self {
            start: Box::new(start),
            started: OnceLock::new(),
        }
    }

    /// The provider, started now if nobody has asked before.
    pub(super) fn get(&self) -> &Started<T> {
        self.started.get_or_init(|| (self.start)())
    }
}
