use std::{fs, process::exit, sync::Arc, time::Instant};

use log::{debug, error, info};
use scylla::{
    load_balancing::DefaultPolicy, prepared_statement::PreparedStatement, transport::Compression,
    ExecutionProfile, Session, SessionBuilder,
};
use tokio::{
    sync::Semaphore,
    task::{self, JoinHandle},
};

use crate::data::source_model::LogEntries;

#[derive(Clone)]
pub struct ScyllaDbService {
    parallelism: usize,
    db_session: Arc<Session>,
    ps: Arc<PreparedStatement>,
}

const INSERT_QUERY: &str = "INSERT INTO datalake.logs (id, ingestion_id, timestamp, user_id, event_type, page_url, ip_address, device_type, browser, os, response_time) VALUES (?,?,?,?,?,?,?,?,?,?,?)";

impl ScyllaDbService {
    pub async fn new(dc: &str, host: &str, db_parallelism: usize, schema_file: &str) -> Self {
        debug!("ScyllaDbService: connecting to {}. DC: {}.", host, dc);
        let policy = Arc::new(DefaultPolicy::default());
        let profile = ExecutionProfile::builder()
            .load_balancing_policy(policy)
            .build();
        let session: Session = SessionBuilder::new()
            .known_node(host)
            .compression(Some(Compression::Lz4))
            .default_execution_profile_handle(profile.into_handle())
            .build()
            .await
            .expect("Error connecting to ScyllaDB");
        info!("ScyllaDbService: connected to {}. DC: {}.", host, dc);

        debug!("ScyllaDbService: creating schema...");
        let schema = fs::read_to_string(schema_file)
            .unwrap_or_else(|_| { panic!("{}", ("Error Reading Schema file".to_owned() + schema_file)) });

        let schema_query = schema.trim().replace('\n', "");

        for q in schema_query.split(';') {
            let query = q.to_owned() + ";";
            if query.len() > 1 {
                info!("Running Query: {}", query);
                if let Err(e) = session
                    .query(query, &[])
                    .await {
                        error!("Query Failed: {:?} {:?}", q, e);
                        exit(1)
                    }
            }
        }

        let mut ps = match session
            .prepare(INSERT_QUERY)
            .await {
                Ok(ps) => ps,
                Err(e) => {
                    error!("Prepared Statement failed: {:?}", e);
                    exit(1)
                }
            };
        ps.set_consistency(scylla::statement::Consistency::Any);

        Self {
            parallelism: db_parallelism,
            db_session: Arc::new(session),
            ps: Arc::new(ps),
        }
    }

    pub async fn insert(&self, entries: LogEntries) -> Result<(), anyhow::Error> {
        let now = Instant::now();
        let sem = Arc::new(Semaphore::new(self.parallelism));
        debug!("ScyllaDbService: insert: saving logs...");
        let mut i = 0;
        let mut handlers: Vec<JoinHandle<_>> = Vec::new();
        for entry in entries {
            let session = self.db_session.clone();
            let prepared = self.ps.clone();
            let permit = sem.clone().acquire_owned().await;
            debug!("insert: creating tasks");
            handlers.push(task::spawn(async move {
                let result = match session.execute(&prepared, entry).await {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(anyhow::Error::msg(format!("Query execution failed: {}", e)))
                    }
                };
                let _permit = permit;
                Ok(result)
            }));
            debug!("insert: tasks created");
            i += 1;
        }
        debug!("ScyllaDbService: insert: Waiting for {i} tasks to complete");

        let mut error_count = 0;
        for thread in handlers {
            match thread.await {
                Err(e) => {
                    error_count += 1;
                    error!("insert: Error executing Query. {:?}", e)
                }
                Ok(r) => match r {
                    Ok(r) => debug!("insert: Query Result: {:?}", r),
                    Err(e) => error!("Thread error: {:?}", e),
                },
            }
        }
        info!(
            "ScyllaDbService: insert: {} insert log tasks completed. Errors: {}. Took: {:.2?}",
            i, error_count, now.elapsed()
        );
        Ok(())
    }
}
