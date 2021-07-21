address 0x2 {
// a distributed key-value map is used to store entry (token_id, address, NonFungibleToken)
// key is the token_id(:vector<u8>), stored in a sorted linked list
// value is a struct 'NonFungibleToken', contains the non fungible token
// the account address of each list node is actually the owner of the token
module NonFungibleToken {
    use 0x2::SimpleSortedLinkedList;
    use Std::Option::{Self, Option};
    use Std::Signer;
    use Std::Vector;

    const NFT_PUBLISHER: address = @0x2;

    struct LimitedMeta has key {
        limited: bool,
        total: u64,
    }

    struct NonFungibleToken<Token> has key, store {
        token: Option<Token>
    }

    struct TokenLock<phantom Token> has key {
    }

    fun lock<Token: store>(account: &signer) {
        move_to<TokenLock<Token>>(account, TokenLock<Token>{});
    }

    fun unlock<Token: store>(account: &signer) acquires TokenLock {
        let sender = Signer::address_of(account);
        let TokenLock<Token> {} = move_from<TokenLock<Token>>(sender);
    }

    public fun initialize<Token: store>(account: &signer, limited: bool, total: u64) {
        let sender = Signer::address_of(account);
        assert(sender == NFT_PUBLISHER, 8000);

        let limited_meta = LimitedMeta {
            limited: limited,
            total: total,
        };
        move_to<LimitedMeta>(account, limited_meta);
        SimpleSortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
    }

    public fun preemptive<Token: store>(account: &signer, nft_service_address: address, token_id: vector<u8>, token: Token):Option<Token> {
        let (exist, location) = Self::find(copy token_id, nft_service_address);
        if (exist) return Option::some(token);

        SimpleSortedLinkedList::add_node<vector<u8>>(account, token_id, location);
        move_to<NonFungibleToken<Token>>(account, NonFungibleToken<Token>{token: Option::some(token)});
        Option::none() //preemptive success
    }

    public fun accept_token<Token: store>(account: &signer) {
        let sender = Signer::address_of(account);
        assert(!exists<NonFungibleToken<Token>>(sender), 8001);
        SimpleSortedLinkedList::empty_node<vector<u8>>(account, Vector::empty());
        move_to<NonFungibleToken<Token>>(account, NonFungibleToken<Token>{token: Option::none()});
    }

    public fun safe_transfer<Token: drop + store>(account: &signer, _nft_service_address: address, token_id: vector<u8>, receiver: address) acquires NonFungibleToken {
        let sender = Signer::address_of(account);
        assert(exists<NonFungibleToken<Token>>(receiver), 8002);
        assert(Option::is_none(&borrow_global<NonFungibleToken<Token>>(receiver).token), 8005);
        assert(Self::get_token_id(sender) == token_id, 8003);
        assert(!exists<TokenLock<Token>>(sender), 8004);

        SimpleSortedLinkedList::move_node_to<vector<u8>>(account, receiver);
        let NonFungibleToken<Token>{ token } = move_from<NonFungibleToken<Token>>(sender);
        let receiver_token_ref_mut = borrow_global_mut<NonFungibleToken<Token>>(receiver);
        receiver_token_ref_mut.token = token;
    }

    public fun get_token_id(addr: address): vector<u8> {
        SimpleSortedLinkedList::get_key_of_node<vector<u8>>(addr)
    }

    public fun find(token_id: vector<u8>, head_address: address): (bool, address) {
        SimpleSortedLinkedList::find<vector<u8>>(token_id, head_address)
    }

    public fun get_nft<Token: store>(account: &signer): NonFungibleToken<Token> acquires NonFungibleToken {
        let sender = Signer::address_of(account);
        assert(exists<NonFungibleToken<Token>>(sender), 8006);
        assert(!exists<TokenLock<Token>>(sender), 8007);
        Self::lock<Token>(account);
        move_from<NonFungibleToken<Token>>(sender)
    }

    public fun put_nft<Token: store>(account: &signer, nft: NonFungibleToken<Token>) acquires TokenLock {
        let sender = Signer::address_of(account);
        assert(exists<TokenLock<Token>>(sender), 8008);
        Self::unlock<Token>(account);
        move_to<NonFungibleToken<Token>>(account, nft)
    }
}
}

/*
//! new-transaction
//! sender: nftservice
module TestNft {
    struct TestNft {}
    public fun new_test_nft(): TestNft {
        TestNft{}
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
// sample for moving Nft into another resource
module MoveNft {
    use {{nftservice}}::NonFungibleToken::{Self, NonFungibleToken};
    use {{nftservice}}::TestNft::TestNft;
    use Std::Signer;

    resource struct MoveNft {
        nft: NonFungibleToken<TestNft>
    }

    public fun move_nft(account: &signer) {
        let nft = NonFungibleToken::get_nft<TestNft>(account);
        move_to<MoveNft>(account, MoveNft{ nft });
    }

    public fun move_back_nft(account: &signer) acquires MoveNft {
        let sender = Signer::address_of(account);
        let MoveNft { nft } = move_from<MoveNft>(sender);
        NonFungibleToken::put_nft<TestNft>(account, nft);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: nftservice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
fun main(account: signer) {
    NonFungibleToken::initialize<TestNft>(account, false, 0);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::{Self, TestNft};
use Std::Hash;
fun main(account: signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let token = TestNft::new_test_nft();
    NonFungibleToken::preemptive<TestNft>(account, {{nftservice}}, token_id, token);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{alice}}::MoveNft;
fun main(account: signer) {
    MoveNft::move_nft(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
fun main(account: signer) {
    NonFungibleToken::accept_token<TestNft>(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
use Std::Hash;
fun main(account: signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    NonFungibleToken::safe_transfer<TestNft>(account, {{nftservice}}, token_id, {{bob}});
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::MoveNft;
fun main(account: signer) {
    MoveNft::move_back_nft(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
use Std::Hash;
fun main(account: signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    NonFungibleToken::safe_transfer<TestNft>(account, {{nftservice}}, token_id, {{bob}});
}
}

// check: "Keep(EXECUTED)"
*/
