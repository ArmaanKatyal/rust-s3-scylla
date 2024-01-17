## Overview
Rust-S3-Scylla is a Rust-based application designed for efficient data ingestion from Amazon S3 datalakes into a ScyllaDB cluster. It exploits Rust's concurrency capabilities to process multiple files in parallel, ensuring fast and reliable data transfer. This solution offers a REST API interface, allowing users to submit requests for ingesting data from specified S3 buckets into ScyllaDB.

## Prerequisites
- Latest stable version of Rust.
- Access to an Amazon S3 bucket.
- A ScyllaDB cluster up and running.
- Network connectivity to both S3 and ScyllaDB.

## Usage
To start ingesting files from S3 into ScyllaDB, send a POST request to the ingestion endpoint:

Endpoint: POST http://localhost:3000/ingest

Request Body:
```json
{
    "ingestion_id": "1",
    "bucket": "your-bucket",
    "files": ["example1.json", "example2.json", "example3.json", "example4.json"]
}
```

## Configuration
Configure the application through config and .env file or environment variables, including:

S3 Access Keys.
ScyllaDB connection details.
See .env.example for configuration format.
