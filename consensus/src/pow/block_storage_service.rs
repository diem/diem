use std::sync::Arc;
use crate::chained_bft::consensusdb::ConsensusDB;
use block_storage_proto::{proto::block_storage::{BlockStorage, create_block_storage, GetBlockByBlockIdResponse as GetBlockByBlockIdResponseProto}};
use grpcio::{EnvBuilder, RpcContext, UnarySink, Server, ServerBuilder};
use libra_config::config::NodeConfig;
use std::convert::TryFrom;
use grpc_helpers::{provide_grpc_response, spawn_service_thread_with_drop_closure, ServerHandle};
use consensus_types::{block::Block};
use crate::pow::payload_ext::BlockPayloadExt;
use network::proto::Block as BlockBytes;
use libra_crypto::HashValue;
use libra_types::explorer::{BlockId, GetBlockSummaryListRequest, BlockSummary, GetBlockSummaryListResponse};
use libra_types::proto::types::{BlockId as BlockIdProto, GetBlockSummaryListRequest as GetBlockSummaryListRequestProto,
                                GetBlockSummaryListResponse as GetBlockSummaryListResponseProto,
                                LatestBlockHeightResponse as LatestBlockHeightResponseProto};

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
    fn get_block_by_block_id(&mut self, ctx: RpcContext,
                                     req: BlockIdProto,
                                     sink: UnarySink<GetBlockByBlockIdResponseProto>) {
        let block_id = BlockId::try_from(req).expect("parse err.");
        let block: Option<Block<BlockPayloadExt>> = self.block_storage.get_block_by_hash(&block_id.id);
        let mut resp = GetBlockByBlockIdResponseProto::default();
        match block {
            Some(b) => {
                resp.block = Some(BlockBytes::try_from(b).expect("to BlockBytes err."));
            },
            None => {},
        }

        provide_grpc_response(Ok(resp), ctx, sink);
    }

    fn get_block_summary_list(&mut self, ctx: RpcContext,
                              req: GetBlockSummaryListRequestProto,
                              sink: UnarySink<GetBlockSummaryListResponseProto>) {
        let block_id = GetBlockSummaryListRequest::try_from(req).expect("to GetBlockSummaryListRequest err.");
        let height = match block_id.block_id {
            Some(id) => {
                let block:Block<BlockPayloadExt> = self.block_storage.get_block_by_hash(&id.id).expect("block not exist.");
                Some(block.round())
            },
            None => None,
        };
        let block_index_vec = self.block_storage.query_block_index(height, 10).expect("query err.");
        let hashs = block_index_vec.iter().map(|block_index| {block_index.id()}).collect();
        let blocks: Option<Vec<Block<BlockPayloadExt>>> = self.block_storage.get_blocks_by_hashs(hashs);
        let block_summary_vec = match blocks {
            Some(b_s) => {
                b_s.iter().map(|b| {
                    block_2_summary(b)
                }).collect()
            },
            None => vec![],
        };
        let resp = GetBlockSummaryListResponse {blocks:block_summary_vec};
        provide_grpc_response(Ok(resp.into()), ctx, sink);
    }

    fn latest_block_height(&mut self, ctx: RpcContext, req: (), sink: UnarySink<LatestBlockHeightResponseProto>) {
        let latest = self.block_storage.latest_height();
        let mut resp = LatestBlockHeightResponseProto::default();
        match latest {
            Some(height) => {
                resp.height = height;
            },
            None => {}
        }

        provide_grpc_response(Ok(resp), ctx, sink);
    }
}

fn block_2_summary(block:&Block<BlockPayloadExt>) -> BlockSummary {
    let payload = block.payload().expect("payload is none.");
    BlockSummary {
        block_id: block.id(),
        height: block.round(),
        parent_id: block.parent_id(),
        accumulator_root_hash: block.quorum_cert().commit_info().executed_state_id(),
        state_root_hash: block.quorum_cert().ledger_info().ledger_info().consensus_data_hash(),
        miner: block.author().expect("author is none."),
        nonce: payload.nonce.clone(),
        target: HashValue::from_slice(payload.target.clone().as_slice()).expect("target to HashValue err."),
        algo: payload.algo,
    }
}