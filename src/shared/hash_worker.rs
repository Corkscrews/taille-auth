use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_trait::async_trait;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use flume;
use rayon::ThreadPoolBuilder;
use std::thread; // Use async_trait for async functions in traits

enum WorkOrder {
  Hash(String),
  Verify(String, String)
}

enum WorkOrderResult {
  Hash(Result<String, BcryptError>),
  Verify(Result<bool, BcryptError>)
}

// Define the Worker struct that implements the Hasher trait
pub struct HashWorker {
  sender: flume::Sender<(flume::Sender<WorkOrderResult>, WorkOrder)>,
}

impl HashWorker {
  pub fn new() -> Self {
    // Uses rayon
    let thread_pool = ThreadPoolBuilder::new().num_threads(2).build().unwrap();

    // Create a channel for communication between async tasks and threads
    let (tx, rx) = flume::bounded::<(flume::Sender<WorkOrderResult>, WorkOrder)>(100);

    // TODO, how to dispatch multiple threads to the same flume::bounded, not sure if this
    // is happening here
    thread_pool.spawn(move || {
    // Spin up a thread pool for CPU-bound tasks
    // thread::spawn(move || {
      while let Ok((response_tx, work_order)) = rx.recv() {
        match work_order {
          WorkOrder::Hash(password) => {
            // Perform the CPU-bound task: Hashing the input data
            let result = hash(password, DEFAULT_COST);
            // Send the result back through the flume channel
            let _ = response_tx.send(WorkOrderResult::Hash(result));
          }
          WorkOrder::Verify(password, hash_password) => {
            // Perform the CPU-bound task: Hashing the input data
            let result = verify(password, &hash_password);
            // Send the result back through the flume channel
            let _ = response_tx.send(WorkOrderResult::Verify(result));
          }
        }
      }
    });

    Self { sender: tx }
  }
}

// Define the Hasher trait
#[async_trait]
pub trait Hasher {
  async fn hash_password(&self, password: &str) -> Result<String, BcryptError>;
  async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, BcryptError>;
}

#[async_trait]
impl Hasher for HashWorker {
  async fn hash_password(&self, password: &str) -> Result<String, BcryptError> {
    // Create a flume channel for this task
    let (response_tx, response_rx) = flume::bounded(1);

    // Send the work order to the thread pool
    let work_order = WorkOrder::Hash(password.to_string());
    self
      .sender
      .send_async((response_tx, work_order))
      .await
      .unwrap();

    // Await the result from the flume channel
    let result = response_rx
      .recv_async()
      .await
      .unwrap();

    match result {
      WorkOrderResult::Hash(hash_password_result) => hash_password_result,
      _ => unreachable!()
    }
  }
  async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, BcryptError> {
    // Create a flume channel for this task
    let (response_tx, response_rx) = flume::bounded(1);

    // Send the work order to the thread pool
    let work_order = WorkOrder::Verify(password.to_string(), hash.to_string());
    self
      .sender
      .send_async((response_tx, work_order))
      .await
      .unwrap();

    // Await the result from the flume channel
    let result = response_rx
      .recv_async()
      .await
      .unwrap();

    match result {
      WorkOrderResult::Verify(password_match_result) => password_match_result,
      _ => unreachable!()
    }
  }
}
