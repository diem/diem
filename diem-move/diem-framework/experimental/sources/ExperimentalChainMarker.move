module ExperimentalFramework::ExperimentalChainMarker {
    // TODO: add other on-chain config friends here
    friend ExperimentalFramework::ExperimentalVersion;

    struct ExperimentalChainMarker has drop {}

    public(friend) fun get_chain_marker(): ExperimentalChainMarker {
        ExperimentalChainMarker {}
    }
}
