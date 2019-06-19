#!/usr/bin/env python3
# Copyright (c) 2014-2019 The Bitcoin Core developers
# Distributed under the MIT software license, see the accompanying
# file COPYING or http://www.opensource.org/licenses/mit-license.php.
"""Test mempool limiting together/eviction with the wallet."""

from decimal import Decimal

from test_framework.test_framework import BitcoinTestFramework
from test_framework.util import assert_equal, assert_greater_than, assert_raises_rpc_error, create_confirmed_utxos, create_lots_of_big_transactions, gen_return_txouts

class MempoolLimitTest(BitcoinTestFramework):
    def set_test_params(self):
        self.setup_clean_chain = True
        self.num_nodes = 1
        self.extra_args = [["-maxmempool=5", "-spendzeroconfchange=0"]]

    def skip_test_if_missing_module(self):
        self.skip_if_no_wallet()

    def run_test(self):
        txouts = gen_return_txouts()
        relayfee = self.nodes[0].getnetworkinfo()['relayfee']

        self.log.info('Check that mempoolminfee is minrelytxfee')
        assert_equal(self.nodes[0].getmempoolinfo()['minrelaytxfee'], Decimal('0.00001000'))
        assert_equal(self.nodes[0].getmempoolinfo()['mempoolminfee'], Decimal('0.00001000'))

        txids = []
        utxos = create_confirmed_utxos(relayfee, self.nodes[0], 91)

        self.log.info('Create a mempool tx that will be evicted')
        us0 = utxos.pop()
        inputs = [{ "txid" : us0["txid"], "vout" : us0["vout"]}]
        outputs = {self.nodes[0].getnewaddress() : 0.0001}
        tx = self.nodes[0].createrawtransaction(inputs, outputs)
        self.nodes[0].settxfee(relayfee) # specifically fund this tx with low fee
        txF = self.nodes[0].fundrawtransaction(tx)
        self.nodes[0].settxfee(0) # return to automatic fee selection
        txFS = self.nodes[0].signrawtransactionwithwallet(txF['hex'])
        txid = self.nodes[0].sendrawtransaction(txFS['hex'])

        relayfee = self.nodes[0].getnetworkinfo()['relayfee']
        base_fee = relayfee*100
        for i in range (3):
            txids.append([])
            txids[i] = create_lots_of_big_transactions(self.nodes[0], txouts, utxos[30*i:30*i+30], 30, (i+1)*base_fee)

        self.log.info('The tx should be evicted by now')
        assert txid not in self.nodes[0].getrawmempool()
        txdata = self.nodes[0].gettransaction(txid)
        assert txdata['confirmations'] ==  0  #confirmation should still be 0

        self.log.info('Check that mempoolminfee is larger than minrelytxfee')
        assert_equal(self.nodes[0].getmempoolinfo()['minrelaytxfee'], Decimal('0.00001000'))
        assert_greater_than(self.nodes[0].getmempoolinfo()['mempoolminfee'], Decimal('0.00001000'))

        self.log.info('Create a mempool tx that will not pass mempoolminfee')
        us0 = utxos.pop()
        inputs = [{ "txid" : us0["txid"], "vout" : us0["vout"]}]
        outputs = {self.nodes[0].getnewaddress() : 0.0001}
        tx = self.nodes[0].createrawtransaction(inputs, outputs)
        # specifically fund this tx with a fee < mempoolminfee, >= than minrelaytxfee
        txF = self.nodes[0].fundrawtransaction(tx, {'feeRate': relayfee})
        txFS = self.nodes[0].signrawtransactionwithwallet(txF['hex'])
        assert_raises_rpc_error(-26, "mempool min fee not met", self.nodes[0].sendrawtransaction, txFS['hex'])

if __name__ == '__main__':
    MempoolLimitTest().main()
