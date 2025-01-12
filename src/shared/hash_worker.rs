use async_trait::async_trait;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use flume;
use rayon::ThreadPool;
use std::sync::Arc; // Use async_trait for async functions in traits

enum WorkOrder {
  Hash(String, flume::Sender<Result<String, BcryptError>>),
  Verify(String, String, flume::Sender<Result<bool, BcryptError>>)
}

// Define the Worker struct that implements the Hasher trait
pub struct HashWorker {
  sender: flume::Sender<WorkOrder>,
}

impl HashWorker {
  pub fn new(thread_pool: ThreadPool, num_threads: u32) -> Self {
    // Create a channel for communication between async tasks and threads
    let (tx, rx) =
      flume::bounded::<WorkOrder>(100);
    let rx = Arc::new(rx);

    // Spin up a thread pool for CPU-bound tasks based on the number of required works.
    for _ in 0..num_threads {
      // Dispatch the run-loop.
      thread_pool.spawn({
        let arc_rx = Arc::clone(&rx);
        move || {
          while let Ok(work_order) = arc_rx.recv() {
            match work_order {
              WorkOrder::Hash(password, response) => {
                let _ = response.send(hash(password, DEFAULT_COST));
              },
              WorkOrder::Verify(password, hashed_password, response) => {
                let _ = response.send(verify(password, &hashed_password));
              }
            };
          }
        }
      });
    }

    Self { sender: tx }
  }
}

// Define the Hasher trait
#[async_trait]
pub trait Hasher {
  async fn hash_password(&self, password: &str) -> Result<String, BcryptError>;
  async fn verify_password(
    &self,
    password: &str,
    hash: &str,
  ) -> Result<bool, BcryptError>;
}

#[async_trait]
impl Hasher for HashWorker {
  async fn hash_password(&self, password: &str) -> Result<String, BcryptError> {
    // Create a flume channel for this task
    let (response_tx, response_rx) = flume::bounded(1);
    self
      .sender
      .send_async(WorkOrder::Hash(password.to_string(), response_tx))
      .await
      .unwrap();
    // Await the result from the flume channel
    response_rx.recv_async().await.unwrap()
  }
  async fn verify_password(
    &self,
    password: &str,
    hash: &str,
  ) -> Result<bool, BcryptError> {
    // Create a flume channel for this task
    let (response_tx, response_rx) = flume::bounded(1);
    self
      .sender
      .send_async(
        WorkOrder::Verify(
          password.to_string(), 
          hash.to_string(), 
          response_tx
        )
      )
      .await
      .unwrap();
    // Await the result from the flume channel
    response_rx.recv_async().await.unwrap()
  }
}
