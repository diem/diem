use 0x0::LibraSystem;
use 0x0::Vector;
// rotate vivian's network address
fun main() {
    let info: vector<LibraSystem::DiscoveryInfo>;
    let info_2: LibraSystem::DiscoveryInfo;
    let addr: address;

    info = Vector::empty<LibraSystem::DiscoveryInfo>();
    Vector::push_back<LibraSystem::DiscoveryInfo>(&mut info, LibraSystem::get_ith_discovery_info(0));
    Vector::push_back<LibraSystem::DiscoveryInfo>(&mut info, LibraSystem::get_ith_discovery_info(1));
    Vector::push_back<LibraSystem::DiscoveryInfo>(&mut info, LibraSystem::get_ith_discovery_info(2));

    info_2 = LibraSystem::get_ith_discovery_info(2);
    addr = *(LibraSystem::get_discovery_address(&info_2));

    0x0::Transaction::assert(LibraSystem::get_discovery_index(&info, move addr) == 2, 98)
}
