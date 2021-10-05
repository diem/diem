module DiemFramework::CoreConfig {
    use Std::Vector;

    struct ConfigAccessPaths has key {
        paths: vector<NamedAccessPath>,
    }

    struct NamedAccessPath has store {
        name: vector<u8>,
        path: vector<u8>,
    }

    public fun initialize(root: &signer, version_config_path: vector<u8>) {
        // TODO: assert genesis
        move_to(
            root,
            ConfigAccessPaths {
                paths: Vector::singleton(NamedAccessPath { name: b"DiemVersion", path: version_config_path })
            }
        );
    }
}
