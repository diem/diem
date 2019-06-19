#!/usr/bin/env python3
# Copyright (c) 2018 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test the avoid_reuse and setwalletflag features."""

from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import (
    assert_equal,
    assert_raises_rpc_error,
    connect_nodes_bi,
)

# TODO: Copied from wallet_groups.py -- should perhaps move into util.py
def assert_approx(v, vexp, vspan=0.00001):
    if v < vexp - vspan:
        raise AssertionError("%s < [%s..%s]" % (str(v), str(vexp - vspan), str(vexp + vspan)))
    if v > vexp + vspan:
        raise AssertionError("%s > [%s..%s]" % (str(v), str(vexp - vspan), str(vexp + vspan)))

def reset_balance(node, discardaddr):
    '''Throw away all owned coins by the node so it gets a balance of 0.'''
    balance = node.getbalance(avoid_reuse=False)
    if balance > 0.5:
        node.sendtoaddress(address=discardaddr, amount=balance, subtractfeefromamount=True, avoid_reuse=False)

def count_unspent(node):
    '''Count the unspent outputs for the given node and return various statistics'''
    r = {
        "total": {
            "count": 0,
            "sum": 0,
        },
        "reused": {
            "count": 0,
            "sum": 0,
        },
    }
    supports_reused = True
    for utxo in node.listunspent(minconf=0):
        r["total"]["count"] += 1
        r["total"]["sum"] += utxo["amount"]
        if supports_reused and "reused" in utxo:
            if utxo["reused"]:
                r["reused"]["count"] += 1
                r["reused"]["sum"] += utxo["amount"]
        else:
            supports_reused = False
    r["reused"]["supported"] = supports_reused
    return r

def assert_unspent(node, total_count=None, total_sum=None, reused_supported=None, reused_count=None, reused_sum=None):
    '''Make assertions about a node's unspent output statistics'''
    stats = count_unspent(node)
    if total_count is not None:
        assert_equal(stats["total"]["count"], total_count)
    if total_sum is not None:
        assert_approx(stats["total"]["sum"], total_sum, 0.001)
    if reused_supported is not None:
        assert_equal(stats["reused"]["supported"], reused_supported)
    if reused_count is not None:
        assert_equal(stats["reused"]["count"], reused_count)
    if reused_sum is not None:
        assert_approx(stats["reused"]["sum"], reused_sum, 0.001)

class AvoidReuseTest(BitcoinTestFramework):

    def set_test_params(self):
        self.setup_clean_chain = False
        self.num_nodes = 2

    def skip_test_if_missing_module(self):
        self.skip_if_no_wallet()

    def run_test(self):
        '''Set up initial chain and run tests defined below'''

        self.test_persistence()
        self.test_immutable()

        self.nodes[0].generate(110)
        self.sync_all()
        reset_balance(self.nodes[1], self.nodes[0].getnewaddress())
        self.test_fund_send_fund_senddirty()
        reset_balance(self.nodes[1], self.nodes[0].getnewaddress())
        self.test_fund_send_fund_send()

    def test_persistence(self):
        '''Test that wallet files persist the avoid_reuse flag.'''
        # Configure node 1 to use avoid_reuse
        self.nodes[1].setwalletflag('avoid_reuse')

        # Flags should be node1.avoid_reuse=false, node2.avoid_reuse=true
        assert_equal(self.nodes[0].getwalletinfo()["avoid_reuse"], False)
        assert_equal(self.nodes[1].getwalletinfo()["avoid_reuse"], True)

        # Stop and restart node 1
        self.stop_node(1)
        self.start_node(1)
        connect_nodes_bi(self.nodes, 0, 1)

        # Flags should still be node1.avoid_reuse=false, node2.avoid_reuse=true
        assert_equal(self.nodes[0].getwalletinfo()["avoid_reuse"], False)
        assert_equal(self.nodes[1].getwalletinfo()["avoid_reuse"], True)

        # Attempting to set flag to its current state should throw
        assert_raises_rpc_error(-8, "Wallet flag is already set to false", self.nodes[0].setwalletflag, 'avoid_reuse', False)
        assert_raises_rpc_error(-8, "Wallet flag is already set to true", self.nodes[1].setwalletflag, 'avoid_reuse', True)

    def test_immutable(self):
        '''Test immutable wallet flags'''
        # Attempt to set the disable_private_keys flag; this should not work
        assert_raises_rpc_error(-8, "Wallet flag is immutable", self.nodes[1].setwalletflag, 'disable_private_keys')

        tempwallet = ".wallet_avoidreuse.py_test_immutable_wallet.dat"

        # Create a wallet with disable_private_keys set; this should work
        self.nodes[1].createwallet(tempwallet, True)
        w = self.nodes[1].get_wallet_rpc(tempwallet)

        # Attempt to unset the disable_private_keys flag; this should not work
        assert_raises_rpc_error(-8, "Wallet flag is immutable", w.setwalletflag, 'disable_private_keys', False)

        # Unload temp wallet
        self.nodes[1].unloadwallet(tempwallet)

    def test_fund_send_fund_senddirty(self):
        '''
        Test the same as test_fund_send_fund_send, except send the 10 BTC with
        the avoid_reuse flag set to false. This means the 10 BTC send should succeed,
        where it fails in test_fund_send_fund_send.
        '''

        fundaddr = self.nodes[1].getnewaddress()
        retaddr = self.nodes[0].getnewaddress()

        self.nodes[0].sendtoaddress(fundaddr, 10)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 1 single, unused 10 btc output
        assert_unspent(self.nodes[1], total_count=1, total_sum=10, reused_supported=True, reused_count=0)

        self.nodes[1].sendtoaddress(retaddr, 5)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 1 single, unused 5 btc output
        assert_unspent(self.nodes[1], total_count=1, total_sum=5, reused_supported=True, reused_count=0)

        self.nodes[0].sendtoaddress(fundaddr, 10)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 2 total outputs (5, 10 btc), one unused (5), one reused (10)
        assert_unspent(self.nodes[1], total_count=2, total_sum=15, reused_count=1, reused_sum=10)

        self.nodes[1].sendtoaddress(address=retaddr, amount=10, avoid_reuse=False)

        # listunspent should show 1 total outputs (5 btc), unused
        assert_unspent(self.nodes[1], total_count=1, total_sum=5, reused_count=0)

        # node 1 should now have about 5 btc left (for both cases)
        assert_approx(self.nodes[1].getbalance(), 5, 0.001)
        assert_approx(self.nodes[1].getbalance(avoid_reuse=False), 5, 0.001)

    def test_fund_send_fund_send(self):
        '''
        Test the simple case where [1] generates a new address A, then
        [0] sends 10 BTC to A.
        [1] spends 5 BTC from A. (leaving roughly 5 BTC useable)
        [0] sends 10 BTC to A again.
        [1] tries to spend 10 BTC (fails; dirty).
        [1] tries to spend 4 BTC (succeeds; change address sufficient)
        '''

        fundaddr = self.nodes[1].getnewaddress()
        retaddr = self.nodes[0].getnewaddress()

        self.nodes[0].sendtoaddress(fundaddr, 10)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 1 single, unused 10 btc output
        assert_unspent(self.nodes[1], total_count=1, total_sum=10, reused_supported=True, reused_count=0)

        self.nodes[1].sendtoaddress(retaddr, 5)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 1 single, unused 5 btc output
        assert_unspent(self.nodes[1], total_count=1, total_sum=5, reused_supported=True, reused_count=0)

        self.nodes[0].sendtoaddress(fundaddr, 10)
        self.nodes[0].generate(1)
        self.sync_all()

        # listunspent should show 2 total outputs (5, 10 btc), one unused (5), one reused (10)
        assert_unspent(self.nodes[1], total_count=2, total_sum=15, reused_count=1, reused_sum=10)

        # node 1 should now have a balance of 5 (no dirty) or 15 (including dirty)
        assert_approx(self.nodes[1].getbalance(), 5, 0.001)
        assert_approx(self.nodes[1].getbalance(avoid_reuse=False), 15, 0.001)

        assert_raises_rpc_error(-6, "Insufficient funds", self.nodes[1].sendtoaddress, retaddr, 10)

        self.nodes[1].sendtoaddress(retaddr, 4)

        # listunspent should show 2 total outputs (1, 10 btc), one unused (1), one reused (10)
        assert_unspent(self.nodes[1], total_count=2, total_sum=11, reused_count=1, reused_sum=10)

        # node 1 should now have about 1 btc left (no dirty) and 11 (including dirty)
        assert_approx(self.nodes[1].getbalance(), 1, 0.001)
        assert_approx(self.nodes[1].getbalance(avoid_reuse=False), 11, 0.001)

if __name__ == '__main__':
    AvoidReuseTest().main()
