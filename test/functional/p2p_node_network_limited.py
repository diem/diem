#!/usr/bin/env python3
# Copyright (c) 2017-2019 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Tests NODE_NETWORK_LIMITED.

Tests that a node configured with -prune=550 signals NODE_NETWORK_LIMITED correctly
and that it responds to getdata requests for blocks correctly:
    - send a block within 288 + 2 of the tip
    - disconnect peers who request blocks older than that."""
from test_framework.messages import CInv, msg_getdata, msg_verack, NODE_BLOOM, NODE_NETWORK_LIMITED, NODE_WITNESS
from test_framework.mininode import P2PInterface, mininode_lock
from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import (
    assert_equal,
    disconnect_nodes,
    connect_nodes_bi,
    wait_until,
)


class P2PIgnoreInv(P2PInterface):
    firstAddrnServices = 0
    def on_inv(self, message):
        # The node will send us invs for other blocks. Ignore them.
        pass
    def on_addr(self, message):
        self.firstAddrnServices = message.addrs[0].nServices
    def wait_for_addr(self, timeout=5):
        test_function = lambda: self.last_message.get("addr")
        wait_until(test_function, timeout=timeout, lock=mininode_lock)
    def send_getdata_for_block(self, blockhash):
        getdata_request = msg_getdata()
        getdata_request.inv.append(CInv(2, int(blockhash, 16)))
        self.send_message(getdata_request)

class NodeNetworkLimitedTest(BitcoinTestFramework):
    def set_test_params(self):
        self.setup_clean_chain = True
        self.num_nodes = 3
        self.extra_args = [['-prune=550', '-addrmantest'], [], []]

    def disconnect_all(self):
        disconnect_nodes(self.nodes[0], 1)
        disconnect_nodes(self.nodes[1], 0)
        disconnect_nodes(self.nodes[2], 1)
        disconnect_nodes(self.nodes[2], 0)
        disconnect_nodes(self.nodes[0], 2)
        disconnect_nodes(self.nodes[1], 2)

    def setup_network(self):
        self.add_nodes(self.num_nodes, self.extra_args)
        self.start_nodes()

    def run_test(self):
        node = self.nodes[0].add_p2p_connection(P2PIgnoreInv())

        expected_services = NODE_BLOOM | NODE_WITNESS | NODE_NETWORK_LIMITED

        self.log.info("Check that node has signalled expected services.")
        assert_equal(node.nServices, expected_services)

        self.log.info("Check that the localservices is as expected.")
        assert_equal(int(self.nodes[0].getnetworkinfo()['localservices'], 16), expected_services)

        self.log.info("Mine enough blocks to reach the NODE_NETWORK_LIMITED range.")
        connect_nodes_bi(self.nodes, 0, 1)
        blocks = self.nodes[1].generatetoaddress(292, self.nodes[1].get_deterministic_priv_key().address)
        self.sync_blocks([self.nodes[0], self.nodes[1]])

        self.log.info("Make sure we can max retrieve block at tip-288.")
        node.send_getdata_for_block(blocks[1])  # last block in valid range
        node.wait_for_block(int(blocks[1], 16), timeout=3)

        self.log.info("Requesting block at height 2 (tip-289) must fail (ignored).")
        node.send_getdata_for_block(blocks[0])  # first block outside of the 288+2 limit
        node.wait_for_disconnect(5)

        self.log.info("Check local address relay, do a fresh connection.")
        self.nodes[0].disconnect_p2ps()
        node1 = self.nodes[0].add_p2p_connection(P2PIgnoreInv())
        node1.send_message(msg_verack())

        node1.wait_for_addr()
        #must relay address with NODE_NETWORK_LIMITED
        assert_equal(node1.firstAddrnServices, 1036)

        self.nodes[0].disconnect_p2ps()
        node1.wait_for_disconnect()

        # connect unsynced node 2 with pruned NODE_NETWORK_LIMITED peer
        # because node 2 is in IBD and node 0 is a NODE_NETWORK_LIMITED peer, sync must not be possible
        connect_nodes_bi(self.nodes, 0, 2)
        try:
            self.sync_blocks([self.nodes[0], self.nodes[2]], timeout=5)
        except:
            pass
        # node2 must remain at height 0
        assert_equal(self.nodes[2].getblockheader(self.nodes[2].getbestblockhash())['height'], 0)

        # now connect also to node 1 (non pruned)
        connect_nodes_bi(self.nodes, 1, 2)

        # sync must be possible
        self.sync_blocks()

        # disconnect all peers
        self.disconnect_all()

        # mine 10 blocks on node 0 (pruned node)
        self.nodes[0].generatetoaddress(10, self.nodes[0].get_deterministic_priv_key().address)

        # connect node1 (non pruned) with node0 (pruned) and check if the can sync
        connect_nodes_bi(self.nodes, 0, 1)

        # sync must be possible, node 1 is no longer in IBD and should therefore connect to node 0 (NODE_NETWORK_LIMITED)
        self.sync_blocks([self.nodes[0], self.nodes[1]])

if __name__ == '__main__':
    NodeNetworkLimitedTest().main()
