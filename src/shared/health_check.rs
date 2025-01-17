use std::{
  sync::{Arc, RwLock},
  time::Duration,
};

use actix_web::rt::spawn;
use actix_web::rt::time::interval;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::database::Database;

#[derive(ToSchema, Clone, Serialize, Deserialize)]
pub struct HealthCheckStats {
  pub database_status: String,
  pub database_name: String,
}

#[automock]
pub trait HealthCheck {
  fn collect(&self) -> Option<HealthCheckStats>;
}

pub struct HealthCheckImpl {
  last_health_check_stats: Arc<RwLock<Option<HealthCheckStats>>>,
}

impl HealthCheckImpl {
  pub fn new<DB: Database + Send + 'static>(database: Arc<DB>) -> Self {
    let database = database.clone();
    let stats_storage: Arc<RwLock<Option<HealthCheckStats>>> =
      Arc::new(RwLock::new(None));

    spawn({
      let stats_storage = stats_storage.clone();
      async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
          interval.tick().await;
          let database_stats = database.stats().await;
          let mut stats = stats_storage.write().unwrap();
          *stats = Some(HealthCheckStats {
            database_status: String::from(if database_stats.connected {
              "connected"
            } else {
              "connecting"
            }),
            database_name: database_stats.name,
          });
        }
      }
    });

    Self {
      last_health_check_stats: stats_storage.clone(),
    }
  }
}

impl HealthCheck for HealthCheckImpl {
  fn collect(&self) -> Option<HealthCheckStats> {
    self
      .last_health_check_stats
      .read()
      .ok()
      .and_then(|stats| stats.clone())
  }
}
