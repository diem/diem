use consensus_types::{
    block::Block, block_data::BlockData, quorum_cert::QuorumCert, timeout::Timeout, vote::Vote,
    vote_proposal::VoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_secure_net::NetworkClient;
use libra_types::epoch_change::EpochChangeProof;
use safety_rules::{
    ConsensusState, Error, RemoteService, SafetyRulesInput, SerializerService, TSafetyRules,
};
use std::sync::{Arc, RwLock};

trait TSerializerClient: Send + Sync {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error>;
}

pub struct LocalService {
    pub serializer_service: Arc<RwLock<SerializerService>>,
}

impl TSerializerClient for LocalService {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.serializer_service
            .write()
            .unwrap()
            .handle_message(input_message)
    }
}

struct RemoteClient {
    network_client: NetworkClient,
}

impl RemoteClient {
    pub fn new(network_client: NetworkClient) -> Self {
        Self { network_client }
    }
}

impl TSerializerClient for RemoteClient {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.network_client.write(&input_message)?;
        let result = self.network_client.read()?;
        Ok(result)
    }
}

pub struct SerializerClient {
    service: Box<dyn TSerializerClient>,
}

impl SerializerClient {
    pub fn new_from_remote_service(remote_service: &dyn RemoteService) -> Self {
        let network_client = NetworkClient::new(remote_service.server_address());
        let service = Box::new(RemoteClient::new(network_client));
        Self { service }
    }

    pub fn new_from_local_service(local_service: LocalService) -> Self {
        Self {
            service: Box::new(local_service),
        }
    }

    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl TSafetyRules for SerializerClient {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        lcs::from_bytes(&response)?
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Initialize(Box::new(proof.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Update(Box::new(qc.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal) -> Result<Vote, Error> {
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        lcs::from_bytes(&response)?
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let response = self.request(SafetyRulesInput::SignProposal(Box::new(block_data)))?;
        lcs::from_bytes(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let response = self.request(SafetyRulesInput::SignTimeout(Box::new(timeout.clone())))?;
        lcs::from_bytes(&response)?
    }
}
