#!/usr/bin/env python3
# Copyright (c) 2014-2019 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test longpolling with getblocktemplate."""

from decimal import Decimal

from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import get_rpc_proxy, random_transaction

import threading

class LongpollThread(threading.Thread):
    def __init__(self, node):
        threading.Thread.__init__(self)
        # query current longpollid
        template = node.getblocktemplate({'rules': ['segwit']})
        self.longpollid = template['longpollid']
        # create a new connection to the node, we can't use the same
        # connection from two threads
        self.node = get_rpc_proxy(node.url, 1, timeout=600, coveragedir=node.coverage_dir)

    def run(self):
        self.node.getblocktemplate({'longpollid': self.longpollid, 'rules': ['segwit']})

class GetBlockTemplateLPTest(BitcoinTestFramework):
    def set_test_params(self):
        self.num_nodes = 2

    def skip_test_if_missing_module(self):
        self.skip_if_no_wallet()

    def run_test(self):
        self.log.info("Warning: this test will take about 70 seconds in the best case. Be patient.")
        self.nodes[0].generate(10)
        template = self.nodes[0].getblocktemplate({'rules': ['segwit']})
        longpollid = template['longpollid']
        # longpollid should not change between successive invocations if nothing else happens
        template2 = self.nodes[0].getblocktemplate({'rules': ['segwit']})
        assert template2['longpollid'] == longpollid

        # Test 1: test that the longpolling wait if we do nothing
        thr = LongpollThread(self.nodes[0])
        thr.start()
        # check that thread still lives
        thr.join(5)  # wait 5 seconds or until thread exits
        assert thr.is_alive()

        # Test 2: test that longpoll will terminate if another node generates a block
        self.nodes[1].generate(1)  # generate a block on another node
        # check that thread will exit now that new transaction entered mempool
        thr.join(5)  # wait 5 seconds or until thread exits
        assert not thr.is_alive()

        # Test 3: test that longpoll will terminate if we generate a block ourselves
        thr = LongpollThread(self.nodes[0])
        thr.start()
        self.nodes[0].generate(1)  # generate a block on another node
        thr.join(5)  # wait 5 seconds or until thread exits
        assert not thr.is_alive()

        # Test 4: test that introducing a new transaction into the mempool will terminate the longpoll
        thr = LongpollThread(self.nodes[0])
        thr.start()
        # generate a random transaction and submit it
        min_relay_fee = self.nodes[0].getnetworkinfo()["relayfee"]
        # min_relay_fee is fee per 1000 bytes, which should be more than enough.
        (txid, txhex, fee) = random_transaction(self.nodes, Decimal("1.1"), min_relay_fee, Decimal("0.001"), 20)
        # after one minute, every 10 seconds the mempool is probed, so in 80 seconds it should have returned
        thr.join(60 + 20)
        assert not thr.is_alive()

if __name__ == '__main__':
    GetBlockTemplateLPTest().main()
