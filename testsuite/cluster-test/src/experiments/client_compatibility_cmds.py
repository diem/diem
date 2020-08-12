#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

import os
import pytest
from secrets import token_bytes
import time
from pylibra import AccountKeyUtils, LibraNetwork, AccountResource, FaucetUtils, _config

# Override the config first
JSON_RPC_URL = os.getenv('JSON_RPC_URL')
FAUCET_URL = os.getenv('FAUCET_URL')
NETWORK_COMPAT = 'compat'
_config.ENDPOINT_CONFIG[NETWORK_COMPAT] = {
    'json-rpc': JSON_RPC_URL,
    'faucet': FAUCET_URL,
}

ASSOC_ADDRESS: str = '0000000000000000000000000a550c18'

API = LibraNetwork(NETWORK_COMPAT)
FAUCET = FaucetUtils(NETWORK_COMPAT)


def wait_for_account_seq(addr_hex: str, seq: int) -> AccountResource:
    num_tries = 0
    while num_tries < 20:
        ar = API.getAccount(addr_hex)
        if ar is not None and ar.sequence >= seq:
            print('Mint complete!')
            return ar
        time.sleep(1)
        num_tries += 1
        print('Waiting for mint')
    raise Exception('Wait for account sequence timed out!')


def mint_and_wait(authkey_hex: str, amount: int, currency: str) -> AccountResource:
    seq = FAUCET.mint(authkey_hex=authkey_hex, amount=amount, identifier=currency)
    print('Submitted mint request')
    return wait_for_account_seq(ASSOC_ADDRESS, seq)


# generate test keys
private_key_hex: str = token_bytes(32).hex()
private_key_bytes: bytes = bytes.fromhex(private_key_hex)
addr_hex: str = AccountKeyUtils.from_private_key(private_key_bytes).address.hex()
authkey_hex: str = AccountKeyUtils.from_private_key(
    private_key_bytes
).authentication_key.hex()

# mint to newly created address
mint_amount = 1_000_000
account = API.getAccount(addr_hex)
if not account:
    mint_and_wait(authkey_hex, mint_amount, 'LBR')
    account = API.getAccount(addr_hex)
    if not account:
        raise Exception(f'Could not create vasp account for auth key {authkey_hex}')
    print(f'Minted account balance: {account.balances}')
    assert account.balances['LBR'] == mint_amount
else:
    raise Exception(f'Account already exists at {addr_hex}')
