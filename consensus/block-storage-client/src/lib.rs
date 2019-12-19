use block_storage_proto::proto::block_storage::BlockStorageClient;
use grpcio::{ChannelBuilder, Environment, EnvBuilder};
use std::sync::Arc;

pub fn make_block_storage_client(
    host: &str,
    port: u16,
    max_receive_len: Option<i32>,
) -> BlockStorageClient {
    let env = EnvBuilder::new().name_prefix("grpc-block_storage-").build();
    let mut builder = ChannelBuilder::new(Arc::new(env))
        .primary_user_agent("grpc/block-storage-read");
    if let Some(m) = max_receive_len {
        builder = builder.max_receive_message_len(m);
    }
    let channel = builder.connect(&format!("{}:{}", host, port));
    BlockStorageClient::new(channel)
}