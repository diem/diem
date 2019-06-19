#!/usr/bin/env python3
# Copyright (c) 2014-2019 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test the invalidateblock RPC."""

from test_framework.test_framework import BitcoinTestFramework
from test_framework.address import ADDRESS_BCRT1_UNSPENDABLE
from test_framework.util import (
    assert_equal,
    connect_nodes_bi,
    wait_until,
)


class InvalidateTest(BitcoinTestFramework):
    def set_test_params(self):
        self.setup_clean_chain = True
        self.num_nodes = 3

    def setup_network(self):
        self.setup_nodes()

    def run_test(self):
        self.log.info("Make sure we repopulate setBlockIndexCandidates after InvalidateBlock:")
        self.log.info("Mine 4 blocks on Node 0")
        self.nodes[0].generatetoaddress(4, self.nodes[0].get_deterministic_priv_key().address)
        assert_equal(self.nodes[0].getblockcount(), 4)
        besthash_n0 = self.nodes[0].getbestblockhash()

        self.log.info("Mine competing 6 blocks on Node 1")
        self.nodes[1].generatetoaddress(6, self.nodes[1].get_deterministic_priv_key().address)
        assert_equal(self.nodes[1].getblockcount(), 6)

        self.log.info("Connect nodes to force a reorg")
        connect_nodes_bi(self.nodes, 0, 1)
        self.sync_blocks(self.nodes[0:2])
        assert_equal(self.nodes[0].getblockcount(), 6)
        badhash = self.nodes[1].getblockhash(2)

        self.log.info("Invalidate block 2 on node 0 and verify we reorg to node 0's original chain")
        self.nodes[0].invalidateblock(badhash)
        assert_equal(self.nodes[0].getblockcount(), 4)
        assert_equal(self.nodes[0].getbestblockhash(), besthash_n0)

        self.log.info("Make sure we won't reorg to a lower work chain:")
        connect_nodes_bi(self.nodes, 1, 2)
        self.log.info("Sync node 2 to node 1 so both have 6 blocks")
        self.sync_blocks(self.nodes[1:3])
        assert_equal(self.nodes[2].getblockcount(), 6)
        self.log.info("Invalidate block 5 on node 1 so its tip is now at 4")
        self.nodes[1].invalidateblock(self.nodes[1].getblockhash(5))
        assert_equal(self.nodes[1].getblockcount(), 4)
        self.log.info("Invalidate block 3 on node 2, so its tip is now 2")
        self.nodes[2].invalidateblock(self.nodes[2].getblockhash(3))
        assert_equal(self.nodes[2].getblockcount(), 2)
        self.log.info("..and then mine a block")
        self.nodes[2].generatetoaddress(1, self.nodes[2].get_deterministic_priv_key().address)
        self.log.info("Verify all nodes are at the right height")
        wait_until(lambda: self.nodes[2].getblockcount() == 3, timeout=5)
        wait_until(lambda: self.nodes[0].getblockcount() == 4, timeout=5)
        wait_until(lambda: self.nodes[1].getblockcount() == 4, timeout=5)

        self.log.info("Verify that we reconsider all ancestors as well")
        blocks = self.nodes[1].generatetoaddress(10, ADDRESS_BCRT1_UNSPENDABLE)
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-1])
        # Invalidate the two blocks at the tip
        self.nodes[1].invalidateblock(blocks[-1])
        self.nodes[1].invalidateblock(blocks[-2])
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-3])
        # Reconsider only the previous tip
        self.nodes[1].reconsiderblock(blocks[-1])
        # Should be back at the tip by now
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-1])

        self.log.info("Verify that we reconsider all descendants")
        blocks = self.nodes[1].generatetoaddress(10, ADDRESS_BCRT1_UNSPENDABLE)
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-1])
        # Invalidate the two blocks at the tip
        self.nodes[1].invalidateblock(blocks[-2])
        self.nodes[1].invalidateblock(blocks[-4])
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-5])
        # Reconsider only the previous tip
        self.nodes[1].reconsiderblock(blocks[-4])
        # Should be back at the tip by now
        assert_equal(self.nodes[1].getbestblockhash(), blocks[-1])


if __name__ == '__main__':
    InvalidateTest().main()
