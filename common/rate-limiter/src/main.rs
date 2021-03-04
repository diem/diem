// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_infallible::Mutex;
use diem_rate_limiter::{
    async_lib::AsyncRateLimiter,
    rate_limit::{Bucket, SharedBucket},
};
use futures::{
    channel::mpsc::channel,
    executor::block_on,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    Future, SinkExt, StreamExt,
};

use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};
use tokio_util::compat::*;

// Parameters for running the tests, change the port if there's a conflict
/// Where to run the test read/write
const ADDRESS: &str = "localhost:7777";
/// Total # of bytes to transfer.  May transfer more, but won't transfer less
const BYTES_TO_TRANSFER: usize = 100_000_000;
/// Expected throughput to be configured on the rate limiter
const ALLOWED_BYTES_PER_SEC: usize = 10_000;
/// Chunk size to write bytes to the stream at a time
const CHUNK_SIZE: usize = 10_000;
/// Buffer size used by Tokio when reading, backpressure only occurs after this is full
// const READ_BUFFER_SIZE: usize = 100;
/// Buffer size used by Tokio when writing
// const WRITE_BUFFER_SIZE: usize = 100;

/// Holder for statistics of a single experiment and reader/writer
struct RunStats {
    pub label: &'static str,
    pub num_actions: usize,
    pub num_bytes: usize,
    pub actions_per_s: f64,
    pub bytes_per_s: f64,
    pub elapsed: Duration,
}

/// A main function for testing rate limiting throughput and backpressure
fn main() {
    diem_logger::DiemLogger::init_for_testing();
    println!("Starting experiments");
    println!(
        "Bytes to test: {}, Expected Throughput(actions/s): {}, Expected Throughput(bytes/s): {}",
        BYTES_TO_TRANSFER,
        ALLOWED_BYTES_PER_SEC as f64 / CHUNK_SIZE as f64,
        ALLOWED_BYTES_PER_SEC
    );

    let runtime = Runtime::new().unwrap();

    run_experiment(
        "No Read or Write Limiting",
        &runtime,
        test_no_rate_limit(BYTES_TO_TRANSFER),
    );
    run_experiment(
        "Read Limited",
        &runtime,
        test_rate_limit_read(BYTES_TO_TRANSFER, ALLOWED_BYTES_PER_SEC),
    );
    run_experiment(
        "Write Limited",
        &runtime,
        test_rate_limit_write(BYTES_TO_TRANSFER, ALLOWED_BYTES_PER_SEC),
    );
    run_experiment(
        "Read & Write Limited",
        &runtime,
        test_rate_limit_read_write(
            BYTES_TO_TRANSFER,
            ALLOWED_BYTES_PER_SEC,
            ALLOWED_BYTES_PER_SEC,
        ),
    );
    run_experiment(
        "Write 2x Read Limited",
        &runtime,
        test_rate_limit_read_write(
            BYTES_TO_TRANSFER,
            ALLOWED_BYTES_PER_SEC,
            ALLOWED_BYTES_PER_SEC.saturating_mul(2),
        ),
    );
    run_experiment(
        "Read 2x Write Limited",
        &runtime,
        test_rate_limit_read_write(
            BYTES_TO_TRANSFER,
            ALLOWED_BYTES_PER_SEC.saturating_mul(2),
            ALLOWED_BYTES_PER_SEC,
        ),
    );
}

/// Runs an experiment and prints out the run stats
fn run_experiment<F: Future<Output = Vec<RunStats>> + 'static + Send>(
    label: &'static str,
    runtime: &Runtime,
    experiment: F,
) {
    println!("\n== {} ==", label);
    let stats = block_on(runtime.spawn(experiment)).expect("Expect experiment to finish");
    println!("-- Stats --");
    for run in stats {
        println!(
            "{} stats\t|\tTotal actions: {}\t|\tTotal bytes: {}\t|\tThroughput(actions/s): {}\t|\tThroughput(bytes/s): {}\t|\tTime elapsed: {:?}",
            run.label, run.num_actions, run.num_bytes, run.actions_per_s, run.bytes_per_s, run.elapsed
        );
    }
}

/// A helper to get stats of a read / write operation
async fn stats<F: Future<Output = (usize, usize)>, Block: FnOnce() -> F>(
    label: &'static str,
    block: Block,
) -> RunStats {
    let start = Instant::now();
    let (num_actions, num_bytes) = block().await;
    let end = Instant::now();

    let elapsed = end - start;
    let elapsed_s = (elapsed.as_millis() as f64) / 1000.0;

    let actions_per_s = num_actions as f64 / elapsed_s;
    let bytes_per_s = num_bytes as f64 / elapsed_s;

    RunStats {
        label,
        num_actions,
        num_bytes,
        actions_per_s,
        bytes_per_s,
        elapsed,
    }
}

/// A generalized test of the rate limiter, allowing for turning off rate limiting on either
/// read or write sides.  `total_test_bytes` is the lower bound of send / receive bytes
async fn test_rate_limiter<
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
    Reader: FnOnce(Compat<TcpStream>) -> R + Send + 'static,
    Writer: FnOnce(Compat<TcpStream>) -> W + Send + 'static,
>(
    total_test_bytes: usize,
    reader: Reader,
    writer: Writer,
) -> Vec<RunStats> {
    // Startup the socket connections first, and the stats line
    let listener = TcpListener::bind(ADDRESS)
        .await
        .expect("Must bind to address");
    let out_stream = TcpStream::connect(ADDRESS)
        .await
        .expect("Must connect to address")
        .compat();

    // Can't do this with tokio 1.0 api
    // Shrink the buffer to increase the affects of backpressure
    // out_stream
    //     .set_send_buffer_size(WRITE_BUFFER_SIZE)
    //     .expect("Should be able to change buffer size");
    let mut writer = writer(out_stream);

    let (stats_sender, mut stats_receiver) = channel(2);
    let mut read_stats = stats_sender.clone();
    let mut write_stats = stats_sender.clone();

    // Ensure we're listening first
    let read_future = async move {
        let (in_stream, _) = listener
            .accept()
            .await
            .expect("Should not have an error connecting");
        let in_stream = in_stream.compat();

        // Can't do this with tokio 1.0 api
        // Shrink the buffer to increase the affects of backpressure
        // in_stream
        //     .set_recv_buffer_size(READ_BUFFER_SIZE)
        //     .expect("Should be able to change buffer size");

        // Read in the expected number of bytes
        let mut reader = reader(in_stream);
        let stats = stats("Read", || async {
            let mut buf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
            let mut num_reads: usize = 0;
            let mut num_bytes: usize = 0;
            loop {
                num_bytes = num_bytes.saturating_add(reader.read(&mut buf).await.unwrap());
                num_reads = num_reads.saturating_add(1);
                if num_bytes >= total_test_bytes {
                    break;
                }
            }
            (num_reads, num_bytes)
        })
        .await;
        read_stats.send(stats).await.unwrap();
    };

    let write_future = async move {
        // Write the expected number of bytes
        let stats = stats("Write", || async {
            let source: [u8; CHUNK_SIZE] = [1; CHUNK_SIZE];
            let mut num_writes: usize = 0;
            let mut num_bytes: usize = 0;
            loop {
                num_bytes = num_bytes.saturating_add(writer.write(&source).await.unwrap());
                num_writes = num_writes.saturating_add(1);
                if num_bytes >= total_test_bytes {
                    break;
                }
            }
            (num_writes, num_bytes)
        })
        .await;
        write_stats.send(stats).await.unwrap();
    };

    // Wait for both to finish
    futures::future::join(read_future, write_future).await;

    // Output statistics from the reader & writer
    let mut results: Vec<RunStats> = vec![];
    for _ in 0u8..2u8 {
        results.push(stats_receiver.next().await.unwrap());
    }
    results
}

fn simple_shared_bucket(label: &'static str, size: usize) -> SharedBucket {
    Arc::new(Mutex::new(Bucket::new(
        label.to_string(),
        String::new(),
        "key".to_string(),
        size,
        size,
        size,
        None,
    )))
}

/// For comparison, no rate limiting
async fn test_no_rate_limit(num_bytes: usize) -> Vec<RunStats> {
    test_rate_limiter(num_bytes, |reader| reader, |writer| writer).await
}

/// Tests backpressure, where only the reader is limited.  Using a socket adds a buffer
/// that causes the backpressure only to occur after a large amount of traffic first. (> 500k bytes)
/// > 55 chunks could cause backpressure, for a proper test > 100 or > 200 give good results.
async fn test_rate_limit_read(num_bytes: usize, throughput: usize) -> Vec<RunStats> {
    let inbound_rate_limiter = simple_shared_bucket("read", throughput);
    test_rate_limiter(
        num_bytes,
        |reader| AsyncRateLimiter::new(reader, Some(inbound_rate_limiter)),
        |writer| writer,
    )
    .await
}

/// Tests only if the writer is rate limited, which should provide a smoother read rate
async fn test_rate_limit_write(num_bytes: usize, throughput: usize) -> Vec<RunStats> {
    let outbound_rate_limiter = simple_shared_bucket("write", throughput);
    test_rate_limiter(
        num_bytes,
        |reader| reader,
        |writer| AsyncRateLimiter::new(writer, Some(outbound_rate_limiter)),
    )
    .await
}

/// Tests if both are rate limited.  Results should be roughly the same in the long term as
/// the other two
async fn test_rate_limit_read_write(
    num_bytes: usize,
    read_throughput: usize,
    write_throughput: usize,
) -> Vec<RunStats> {
    let inbound_rate_limiter = simple_shared_bucket("read", read_throughput);
    let outbound_rate_limiter = simple_shared_bucket("write", write_throughput);
    test_rate_limiter(
        num_bytes,
        |reader| AsyncRateLimiter::new(reader, Some(inbound_rate_limiter)),
        |writer| AsyncRateLimiter::new(writer, Some(outbound_rate_limiter)),
    )
    .await
}
