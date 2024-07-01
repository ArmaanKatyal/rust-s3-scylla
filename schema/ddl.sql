CREATE KEYSPACE IF NOT EXISTS datalake WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
CREATE TABLE IF NOT EXISTS datalake.logs (
    id text,
    ingestion_id text,
    timestamp text,
    user_id bigint,
    event_type text,
    page_url text,
    ip_address text,
    device_type text,
    browser text,
    os text,
    response_time double,
    PRIMARY KEY (id, ingestion_id, timestamp)
) WITH comment = 'logs Table' AND caching = {'enabled': 'true'}
    AND compression = {'sstable_compression': 'LZ4Compressor'}
    AND CLUSTERING ORDER BY (ingestion_id ASC, timestamp DESC);
