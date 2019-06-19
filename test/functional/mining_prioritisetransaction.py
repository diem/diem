#!/usr/bin/env python3
# Copyright (c) 2015-2019 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test the prioritisetransaction mining RPC."""

import time

from test_framework.messages import COIN, MAX_BLOCK_BASE_SIZE
from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import assert_equal, assert_raises_rpc_error, create_confirmed_utxos, create_lots_of_big_transactions, gen_return_txouts

class PrioritiseTransactionTest(BitcoinTestFramework):
    def set_test_params(self):
        self.setup_clean_chain = True
        self.num_nodes = 2
        self.extra_args = [["-printpriority=1"], ["-printpriority=1"]]

    def skip_test_if_missing_module(self):
        self.skip_if_no_wallet()

    def run_test(self):
        # Test `prioritisetransaction` required parameters
        assert_raises_rpc_error(-1, "prioritisetransaction", self.nodes[0].prioritisetransaction)
        assert_raises_rpc_error(-1, "prioritisetransaction", self.nodes[0].prioritisetransaction, '')
        assert_raises_rpc_error(-1, "prioritisetransaction", self.nodes[0].prioritisetransaction, '', 0)

        # Test `prioritisetransaction` invalid extra parameters
        assert_raises_rpc_error(-1, "prioritisetransaction", self.nodes[0].prioritisetransaction, '', 0, 0, 0)

        # Test `prioritisetransaction` invalid `txid`
        assert_raises_rpc_error(-8, "txid must be of length 64 (not 3, for 'foo')", self.nodes[0].prioritisetransaction, txid='foo', fee_delta=0)
        assert_raises_rpc_error(-8, "txid must be hexadecimal string (not 'Zd1d4e24ed99057e84c3f80fd8fbec79ed9e1acee37da269356ecea000000000')", self.nodes[0].prioritisetransaction, txid='Zd1d4e24ed99057e84c3f80fd8fbec79ed9e1acee37da269356ecea000000000', fee_delta=0)

        # Test `prioritisetransaction` invalid `dummy`
        txid = '1d1d4e24ed99057e84c3f80fd8fbec79ed9e1acee37da269356ecea000000000'
        assert_raises_rpc_error(-1, "JSON value is not a number as expected", self.nodes[0].prioritisetransaction, txid, 'foo', 0)
        assert_raises_rpc_error(-8, "Priority is no longer supported, dummy argument to prioritisetransaction must be 0.", self.nodes[0].prioritisetransaction, txid, 1, 0)

        # Test `prioritisetransaction` invalid `fee_delta`
        assert_raises_rpc_error(-1, "JSON value is not an integer as expected", self.nodes[0].prioritisetransaction, txid=txid, fee_delta='foo')

        self.txouts = gen_return_txouts()
        self.relayfee = self.nodes[0].getnetworkinfo()['relayfee']

        utxo_count = 90
        utxos = create_confirmed_utxos(self.relayfee, self.nodes[0], utxo_count)
        base_fee = self.relayfee*100 # our transactions are smaller than 100kb
        txids = []

        # Create 3 batches of transactions at 3 different fee rate levels
        range_size = utxo_count // 3
        for i in range(3):
            txids.append([])
            start_range = i * range_size
            end_range = start_range + range_size
            txids[i] = create_lots_of_big_transactions(self.nodes[0], self.txouts, utxos[start_range:end_range], end_range - start_range, (i+1)*base_fee)

        # Make sure that the size of each group of transactions exceeds
        # MAX_BLOCK_BASE_SIZE -- otherwise the test needs to be revised to create
        # more transactions.
        mempool = self.nodes[0].getrawmempool(True)
        sizes = [0, 0, 0]
        for i in range(3):
            for j in txids[i]:
                assert j in mempool
                sizes[i] += mempool[j]['vsize']
            assert sizes[i] > MAX_BLOCK_BASE_SIZE  # Fail => raise utxo_count

        # add a fee delta to something in the cheapest bucket and make sure it gets mined
        # also check that a different entry in the cheapest bucket is NOT mined
        self.nodes[0].prioritisetransaction(txid=txids[0][0], fee_delta=int(3*base_fee*COIN))

        self.nodes[0].generate(1)

        mempool = self.nodes[0].getrawmempool()
        self.log.info("Assert that prioritised transaction was mined")
        assert txids[0][0] not in mempool
        assert txids[0][1] in mempool

        high_fee_tx = None
        for x in txids[2]:
            if x not in mempool:
                high_fee_tx = x

        # Something high-fee should have been mined!
        assert high_fee_tx is not None

        # Add a prioritisation before a tx is in the mempool (de-prioritising a
        # high-fee transaction so that it's now low fee).
        self.nodes[0].prioritisetransaction(txid=high_fee_tx, fee_delta=-int(2*base_fee*COIN))

        # Add everything back to mempool
        self.nodes[0].invalidateblock(self.nodes[0].getbestblockhash())

        # Check to make sure our high fee rate tx is back in the mempool
        mempool = self.nodes[0].getrawmempool()
        assert high_fee_tx in mempool

        # Now verify the modified-high feerate transaction isn't mined before
        # the other high fee transactions. Keep mining until our mempool has
        # decreased by all the high fee size that we calculated above.
        while (self.nodes[0].getmempoolinfo()['bytes'] > sizes[0] + sizes[1]):
            self.nodes[0].generate(1)

        # High fee transaction should not have been mined, but other high fee rate
        # transactions should have been.
        mempool = self.nodes[0].getrawmempool()
        self.log.info("Assert that de-prioritised transaction is still in mempool")
        assert high_fee_tx in mempool
        for x in txids[2]:
            if (x != high_fee_tx):
                assert x not in mempool

        # Create a free transaction.  Should be rejected.
        utxo_list = self.nodes[0].listunspent()
        assert len(utxo_list) > 0
        utxo = utxo_list[0]

        inputs = []
        outputs = {}
        inputs.append({"txid" : utxo["txid"], "vout" : utxo["vout"]})
        outputs[self.nodes[0].getnewaddress()] = utxo["amount"]
        raw_tx = self.nodes[0].createrawtransaction(inputs, outputs)
        tx_hex = self.nodes[0].signrawtransactionwithwallet(raw_tx)["hex"]
        tx_id = self.nodes[0].decoderawtransaction(tx_hex)["txid"]

        # This will raise an exception due to min relay fee not being met
        assert_raises_rpc_error(-26, "min relay fee not met", self.nodes[0].sendrawtransaction, tx_hex)
        assert tx_id not in self.nodes[0].getrawmempool()

        # This is a less than 1000-byte transaction, so just set the fee
        # to be the minimum for a 1000-byte transaction and check that it is
        # accepted.
        self.nodes[0].prioritisetransaction(txid=tx_id, fee_delta=int(self.relayfee*COIN))

        self.log.info("Assert that prioritised free transaction is accepted to mempool")
        assert_equal(self.nodes[0].sendrawtransaction(tx_hex), tx_id)
        assert tx_id in self.nodes[0].getrawmempool()

        # Test that calling prioritisetransaction is sufficient to trigger
        # getblocktemplate to (eventually) return a new block.
        mock_time = int(time.time())
        self.nodes[0].setmocktime(mock_time)
        template = self.nodes[0].getblocktemplate({'rules': ['segwit']})
        self.nodes[0].prioritisetransaction(txid=tx_id, fee_delta=-int(self.relayfee*COIN))
        self.nodes[0].setmocktime(mock_time+10)
        new_template = self.nodes[0].getblocktemplate({'rules': ['segwit']})

        assert template != new_template

if __name__ == '__main__':
    PrioritiseTransactionTest().main()
