use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub struct Executor {
    count: Arc<AtomicUsize>,
}

impl Executor {
    #[inline]
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let counter = AtomicCounter::new(&self.count);
        thread::spawn(move || {
            let _counter = counter;
            f()
        });
    }

    #[allow(dead_code)]
    pub fn join(&self) {
        while self.count.load(Ordering::Acquire) > 0 {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

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
