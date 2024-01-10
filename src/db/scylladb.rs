use std::{fs, sync::Arc};

use log::info;
use scylla::{
    load_balancing::DefaultPolicy, prepared_statement::PreparedStatement, transport::Compression,
    ExecutionProfile, Session, SessionBuilder,
};

#[derive(Clone)]
#[allow(dead_code)]
pub struct ScyllaDbService {
    parallelism: usize,
    db_session: Arc<Session>,
    ps: Arc<PreparedStatement>,
}

const INSERT_QUERY: &str = "INSERT INTO datalake.logs (id, ingestion_id, timestamp, user_id, event_type, page_url, ip_address, device_type, browser, os, response_time) VALUES (?,?,?,?,?,?,?,?,?,?,?)";

impl ScyllaDbService {
    pub async fn new(dc: String, host: String, db_parallelism: usize, schema_file: String) -> Self {
        info!("ScyllaDbService: connecting to {}. DC: {}.", host, dc);
        let policy = Arc::new(DefaultPolicy::default());
        let profile = ExecutionProfile::builder()
            .load_balancing_policy(policy)
            .build();
        let session: Session = SessionBuilder::new()
            .known_node(host.clone())
            .compression(Some(Compression::Lz4))
            .default_execution_profile_handle(profile.into_handle())
            .build()
            .await
            .expect("Error connecting to ScyllaDB");
        info!("ScyllaDbService: connected to {}. DC: {}.", host, dc);

        info!("ScyllaDbService: creating schema...");
        let schema = fs::read_to_string(&schema_file)
            .expect(("Error Reading Schema file".to_owned() + schema_file.as_str()).as_str());

        let schema_query = schema.trim().replace("\n", "");

        for q in schema_query.split(";") {
            let query = q.to_owned() + ";";
            if query.len() > 1 {
                info!("Running Query: {}", query);
                session
                    .query(query, &[])
                    .await
                    .expect("Error creating schema!");
            }
        }

        let mut ps = session
            .prepare(INSERT_QUERY)
            .await
            .expect("Error preparing query!");
        ps.set_consistency(scylla::statement::Consistency::Any);

        Self {
            parallelism: db_parallelism,
            db_session: Arc::new(session),
            ps: Arc::new(ps),
        }
    }
}
