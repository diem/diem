#!/usr/bin/env python3
# Copyright (c) 2018 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test the getblockfilter RPC."""

from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import (
    assert_equal, assert_is_hex_string, assert_raises_rpc_error,
    connect_nodes, disconnect_nodes, sync_blocks
    )

FILTER_TYPES = ["basic"]

class GetBlockFilterTest(BitcoinTestFramework):
    def set_test_params(self):
        self.setup_clean_chain = True
        self.num_nodes = 2
        self.extra_args = [["-blockfilterindex"], []]

    def run_test(self):
        # Create two chains by disconnecting nodes 0 & 1, mining, then reconnecting
        disconnect_nodes(self.nodes[0], 1)

        self.nodes[0].generate(3)
        self.nodes[1].generate(4)

        assert_equal(self.nodes[0].getblockcount(), 3)
        chain0_hashes = [self.nodes[0].getblockhash(block_height) for block_height in range(4)]

        # Reorg node 0 to a new chain
        connect_nodes(self.nodes[0], 1)
        sync_blocks(self.nodes)

        assert_equal(self.nodes[0].getblockcount(), 4)
        chain1_hashes = [self.nodes[0].getblockhash(block_height) for block_height in range(4)]

        # Test getblockfilter returns a filter for all blocks and filter types on active chain
        for block_hash in chain1_hashes:
            for filter_type in FILTER_TYPES:
                result = self.nodes[0].getblockfilter(block_hash, filter_type)
                assert_is_hex_string(result['filter'])

        # Test getblockfilter returns a filter for all blocks and filter types on stale chain
        for block_hash in chain0_hashes:
            for filter_type in FILTER_TYPES:
                result = self.nodes[0].getblockfilter(block_hash, filter_type)
                assert_is_hex_string(result['filter'])

        # Test getblockfilter with unknown block
        bad_block_hash = "0123456789abcdef" * 4
        assert_raises_rpc_error(-5, "Block not found", self.nodes[0].getblockfilter, bad_block_hash, "basic")

        # Test getblockfilter with undefined filter type
        genesis_hash = self.nodes[0].getblockhash(0)
        assert_raises_rpc_error(-5, "Unknown filtertype", self.nodes[0].getblockfilter, genesis_hash, "unknown")

if __name__ == '__main__':
    GetBlockFilterTest().main()
