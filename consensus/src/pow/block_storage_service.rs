use std::sync::Arc;
use crate::chained_bft::consensusdb::ConsensusDB;
use block_storage_proto::proto::block_storage::{BlockStorage, EmptyResponse, create_block_storage,
                                                GetBlockSummaryByBlockIdRequest, GetBlockSummaryByBlockIdResponse,
                                                GetBlockSummaryListRequest, GetBlockSummaryListResponse};
use grpcio::{EnvBuilder, RpcContext, UnarySink, Server, ServerBuilder};
use libra_config::config::NodeConfig;

pub fn make_block_storage_service(node_config: &NodeConfig, block_store: &Arc<ConsensusDB>) -> Server {
    let env = Arc::new(
        EnvBuilder::new()
            .name_prefix("grpc-block-storage-")
            .build(),
    );
    let handle = BlockStorageService::new(Arc::clone(block_store));
    let service = create_block_storage(handle);
    ServerBuilder::new(env.clone())
        .register_service(service)
        .bind(node_config.consensus.consensus_rpc_address.clone(),
              node_config.consensus.consensus_rpc_port)
        .build()
        .expect("Unable to create grpc server")
}

#[derive(Clone)]
pub(crate) struct BlockStorageService {
    block_storage: Arc<ConsensusDB>,
}

impl BlockStorageService {
    pub fn new(db: Arc<ConsensusDB>) -> Self {
        BlockStorageService{block_storage:db}
    }
}

impl BlockStorage for BlockStorageService {
    fn test(&mut self, ctx: RpcContext, req: EmptyResponse, sink: UnarySink<EmptyResponse>) {
        unimplemented!()
    }

    fn get_block_summary_by_block_id(&mut self, ctx: RpcContext,
                                     req: GetBlockSummaryByBlockIdRequest,
                                     sink: UnarySink<GetBlockSummaryByBlockIdResponse>) {
        unimplemented!()
    }

    fn get_block_summary_list(&mut self, ctx: RpcContext,
                              req: GetBlockSummaryListRequest,
                              sink: UnarySink<GetBlockSummaryListResponse>) {
        unimplemented!()
    }
}