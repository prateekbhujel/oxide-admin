use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub trait Job: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self) -> Result<(), String>;
}

pub struct ClosureJob<F>
where
    F: Fn() -> Result<(), String> + Send + Sync,
{
    pub job_name: String,
    pub handler: F,
}

impl<F> Job for ClosureJob<F>
where
    F: Fn() -> Result<(), String> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.job_name
    }

    fn handle(&self) -> Result<(), String> {
        (self.handler)()
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    pub pending: usize,
    pub processed: usize,
    pub failed: usize,
}

#[derive(Clone)]
pub struct JobQueue {
    jobs: Arc<Mutex<VecDeque<Box<dyn Job>>>>,
    processed: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(VecDeque::new())),
            processed: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn dispatch(&self, job: impl Job + 'static) {
        let mut queue = self.jobs.lock().unwrap();
        queue.push_back(Box::new(job));
    }

    pub fn dispatch_fn<F>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.dispatch(ClosureJob {
            job_name: name.into(),
            handler,
        });
    }

    pub fn work_one(&self) -> Option<Result<String, String>> {
        let job = {
            let mut queue = self.jobs.lock().unwrap();
            queue.pop_front()
        }?;

        let name = job.name().to_string();
        match job.handle() {
            Ok(()) => {
                self.processed.fetch_add(1, Ordering::SeqCst);
                Some(Ok(name))
            }
            Err(e) => {
                self.failed.fetch_add(1, Ordering::SeqCst);
                Some(Err(format!("Job '{name}' failed: {e}")))
            }
        }
    }

    pub fn work_all(&self) -> usize {
        let mut count = 0;
        while let Some(_) = self.work_one() {
            count += 1;
        }
        count
    }

    pub fn stats(&self) -> QueueStats {
        let pending = self.jobs.lock().unwrap().len();
        QueueStats {
            pending,
            processed: self.processed.load(Ordering::SeqCst),
            failed: self.failed.load(Ordering::SeqCst),
        }
    }
}
