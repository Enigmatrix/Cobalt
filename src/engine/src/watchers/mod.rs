mod window_closed;

pub use window_closed::*;

use tokio::stream::Stream;

trait Watcher<T> {
    type Stream;
    fn events(&self) -> &Self::Stream;
}
