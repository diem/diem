use block_storage_proto::proto::block_storage::BlockStorageClient;
use grpcio::{ChannelBuilder, Environment};
use std::sync::Arc;

fn make_clients(
    env: Arc<Environment>,
    host: &str,
    port: u16,
    client_type: &str,
    max_receive_len: Option<i32>,
) -> BlockStorageClient {
    let mut builder = ChannelBuilder::new(env.clone())
        .primary_user_agent(format!("grpc/block-storage-{}", client_type).as_str());
    if let Some(m) = max_receive_len {
        builder = builder.max_receive_message_len(m);
    }
    let channel = builder.connect(&format!("{}:{}", host, port));
    BlockStorageClient::new(channel)
}