module CoreFramework::ChainMarker {
    // TODO: remove dependency on DiemFramework
    use DiemFramework::DiemTimestamp;
    use Std::Errors;
    use Std::Signer;

    /// Invalid chain marker was supplied at genesis
    const EINVALID_CHAINMARKER: u64 = 0;
    /// Chain marker was already published at genesis
    const ECHAINMARKER_ALREADY_PUBLISHED: u64 = 1;

    struct SingletonMarker has key {}

    struct ChainMarker<phantom Marker> has key {
        config_address: address,
    }

    public fun initialize<Marker>(
        chain_marker_signer: &signer,
        config_address: address,
    )  {
        let addr = Signer::address_of(chain_marker_signer);
        DiemTimestamp::assert_genesis();
        assert!(
            addr == @ChainMarker,
            Errors::invalid_argument(EINVALID_CHAINMARKER)
        );
        assert!(!exists<SingletonMarker>(addr), Errors::invalid_state(ECHAINMARKER_ALREADY_PUBLISHED));
        move_to(chain_marker_signer, SingletonMarker{});
        move_to(chain_marker_signer, ChainMarker<Marker> {
            config_address,
        })
    }
}
