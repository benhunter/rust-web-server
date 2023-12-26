use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::RecvError;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    counter: Option<mpsc::Receiver<String>>,
    pub job_count: usize,
}

impl ThreadPool {
    pub fn updated_job_count(&mut self) {
        let message = self.counter.as_ref().unwrap().recv();
        match message {
            Ok(message) => {
                self.job_count += 1;
                println!("Job count {0}", self.job_count);
            }
            Err(_) => {
                println!("Error from job counter");
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let (job_count_sender, job_count_receiver) = mpsc::channel();
        let job_count_sender = Arc::new(Mutex::new(job_count_sender));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone(), job_count_sender.clone()));
        }

        ThreadPool { workers, sender: Some(sender), counter: Some(job_count_receiver), job_count: 0 }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
    // job_count_sender: Option<mpsc::Sender<String>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, job_count_sender: Arc<Mutex<mpsc::Sender<String>>>) -> Worker {
        let thread = std::thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job_count_sender.lock().expect("Locking job_count").send(String::from("job")).expect("sending job");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
            // job_count_sender,
        }
    }
}