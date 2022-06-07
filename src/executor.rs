use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct AtomicCounter(Arc<AtomicUsize>);

impl AtomicCounter {
    fn new(count: &Arc<AtomicUsize>) -> Self {
        count.fetch_add(1, Ordering::Relaxed);
        AtomicCounter(Arc::clone(count))
    }
}

impl Drop for AtomicCounter {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

/// Executes a function in either a thread of a thread pool
pub enum Executor {
    Threaded { count: Arc<AtomicUsize> },
    Pooled { pool: threadpool::ThreadPool },
}
impl Executor {
    /// `size` must be greater than zero or the call to `ThreadPool::new` will panic.
    pub fn with_size(size: usize) -> Self {
        let pool = threadpool::ThreadPool::new(size);
        Executor::Pooled { pool }
    }

    #[inline]
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        match *self {
            Executor::Threaded { ref count } => {
                let counter = AtomicCounter::new(count);
                thread::spawn(move || {
                    let _counter = counter;
                    f()
                });
            }
            Executor::Pooled { ref pool } => {
                pool.execute(f);
            }
        }
    }

    pub fn join(&self) {
        match *self {
            Executor::Threaded { ref count } => {
                while count.load(Ordering::Acquire) > 0 {
                    thread::sleep(Duration::from_millis(100));
                }
            }
            Executor::Pooled { ref pool } => {
                pool.join();
            }
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Executor::Threaded {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }
}
