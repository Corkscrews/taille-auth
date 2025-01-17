use async_trait::async_trait;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
// use scrypt::{
//   password_hash::{
//       rand_core::OsRng,
//       PasswordHash, PasswordHasher, PasswordVerifier, SaltString
//   },
//   Scrypt
// };
use flume;
use rayon::ThreadPool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashWorkerError {
  #[error("Bcrypt error: {0}")]
  Bcrypt(#[from] BcryptError),
  // #[error("Scrypt error: {0}")]
  // Scrypt(#[from] scrypt::password_hash::Error),
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
                // let salt = SaltString::generate(&mut OsRng);
                // let _ = response.send(
                //   Scrypt.hash_password(password.as_bytes(), &salt)
                //     .map(|result| result.to_string())
                //     .map_err(HashWorkerError::from)
                // );
              }
              WorkOrder::Verify(password, hashed_password, response) => {
                let _ = response.send(
                  verify(password, &hashed_password)
                    .map_err(HashWorkerError::from),
                );
                // let result = PasswordHash::new(&hashed_password)
                //   .map_err(HashWorkerError::from)
                //   .map(|parsed_hash| {
                //       Scrypt.verify_password(password.as_bytes(), &parsed_hash)
                //         .map(|_| true)
                //         .unwrap_or(false)
                //   });
                // let _ = response.send(result);
              }
            };
          }
        }
      });
    }

    Self { sender: tx }
  }
}

use mockall::automock;

#[automock]
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

#[cfg(test)]
mod tests {
  use super::*;
  use fake::{faker::internet::en::Password, Fake};
  use rayon::ThreadPoolBuilder;

  #[actix_web::test]
  async fn test_hash_and_verify_password() {
    // Create a thread pool with 4 threads
    let thread_pool = ThreadPoolBuilder::new()
      .num_threads(4)
      .build()
      .expect("Failed to create thread pool");

    // Initialize the HashWorker with 4 threads
    let hash_worker = HashWorker::new(thread_pool, 4);

    // Test data
    let password = Password(12..13).fake::<String>();

    // Hash the password
    let hashed_password = hash_worker
      .hash_password(&password)
      .await
      .expect("Hashing failed");

    // Verify the hashed password
    let is_valid = hash_worker
      .verify_password(&password, &hashed_password)
      .await
      .expect("Verification failed");

    // Assert that the hash is valid
    assert!(is_valid, "The password verification failed");

    // Test invalid password
    let invalid_password = "wrong_password";
    let is_invalid = hash_worker
      .verify_password(invalid_password, &hashed_password)
      .await
      .expect("Verification failed for invalid password");

    // Assert that the verification fails for an incorrect password
    assert!(!is_invalid, "The password verification should have failed");
  }
}
