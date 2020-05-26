

struct NetworkNodeConfig {
     signing_public_key: Ed25519PublicKey,
     network_identity_public_key: x25519::PublicKey,
     network_address: RawNetworkAddress,
}

struct NetworkMembershipConfig {
    network_id : string,
    nodes: Map<AccountdAddress, NetworkNodeConfig>,
}

struct NetworkOperationalConfig {

}