use async_trait::async_trait;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use flume;
use rayon::ThreadPool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashWorkerError {
  #[error("Bcrypt error: {0}")]
  Bcrypt(#[from] BcryptError),
  #[error("Channel send error")]
  Send,
  #[error("Channel receive error")]
  Receive,
}

enum WorkOrder {
  Hash(String, flume::Sender<Result<String, HashWorkerError>>),
  Verify(String, String, flume::Sender<Result<bool, HashWorkerError>>),
}

// Define the Worker struct that implements the Hasher trait
pub struct HashWorker {
  sender: flume::Sender<WorkOrder>,
}

impl HashWorker {
  pub fn new(thread_pool: ThreadPool, num_threads: u32) -> Self {
    // Arbitrary number of available channels for processing hash requests. Since each 
    // hashing operation takes at least 1 second to complete, the channel capacity is set
    // to allow up to 3 seconds' worth of requests to queue, ensuring efficient throughput 
    // and minimal blocking.
    let channels_capacity = num_threads * 3;
    // Create a channel for communication between async tasks and threads
    let (tx, rx) = flume::bounded::<WorkOrder>(channels_capacity as usize);
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
                let _ = response.send(
                  hash(password, DEFAULT_COST).map_err(HashWorkerError::from),
                );
              }
              WorkOrder::Verify(password, hashed_password, response) => {
                let _ = response.send(
                  verify(password, &hashed_password)
                    .map_err(HashWorkerError::from),
                );
              }
            };
          }
        }
      });
    }

    Self { sender: tx }
  }
}

#[async_trait]
pub trait Hasher {
  async fn hash_password(
    &self,
    password: &str,
  ) -> Result<String, HashWorkerError>;
  async fn verify_password(
    &self,
    password: &str,
    hash: &str,
  ) -> Result<bool, HashWorkerError>;
}

#[async_trait]
impl Hasher for HashWorker {
  async fn hash_password(
    &self,
    password: &str,
  ) -> Result<String, HashWorkerError> {
    let (response_tx, response_rx) = flume::bounded(1);
    self
      .sender
      .send_async(WorkOrder::Hash(password.to_string(), response_tx))
      .await
      .map_err(|_| HashWorkerError::Send)?;

    response_rx
      .recv_async()
      .await
      .map_err(|_| HashWorkerError::Receive)?
  }

  async fn verify_password(
    &self,
    password: &str,
    hash: &str,
  ) -> Result<bool, HashWorkerError> {
    let (response_tx, response_rx) = flume::bounded(1);
    self
      .sender
      .send_async(WorkOrder::Verify(
        password.to_string(),
        hash.to_string(),
        response_tx,
      ))
      .await
      .map_err(|_| HashWorkerError::Send)?;

    response_rx
      .recv_async()
      .await
      .map_err(|_| HashWorkerError::Receive)?
  }
}
