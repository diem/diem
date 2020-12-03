"""
Simple faucet server
Proxies mint requests to local client that owns association keys
"""
import decimal
import os
import platform
import random
import re
import sys

import flask
import pexpect


MAX_MINT = 10 ** 19  # 10 trillion diems


def create_client():
    if application.client is None or not application.client.isalive():
        # If we have comma separated list take a random one
        ac_hosts = os.environ['AC_HOST'].split(',')
        ac_host = random.choice(ac_hosts)
        ac_port = os.environ['AC_PORT']
        url = "http://{}:{}".format(ac_host, ac_port)
        waypoint = open("/opt/diem/etc/waypoint.txt", "r").readline()
        chain_id = os.environ['CFG_CHAIN_ID']

        print("Connecting to ac on: {}".format(url))
        cmd = "/opt/diem/bin/cli --url {} -m {} --waypoint {} --chain-id {}".format(
            url,
            "/opt/diem/etc/mint.key",
            waypoint,
            chain_id)

        application.client = pexpect.spawn(cmd)
        application.client.delaybeforesend = 1.0
        application.client.expect("Please, input commands")


application = flask.Flask(__name__)
application.client = None
print(sys.version, platform.python_version())
create_client()


@application.route("/", methods=('POST',))
@application.route("/mint", methods=('POST',))
def send_transaction():
    auth_key = flask.request.args['auth_key']

    # Return immediately if auth_key is invalid
    if re.match('^[a-f0-9]{64}$', auth_key) is None:
        return 'Malformed auth_key', 400

    try:
        amount = decimal.Decimal(flask.request.args['amount'])
    except decimal.InvalidOperation:
        return 'Bad amount', 400

    if amount > MAX_MINT:
        return 'Exceeded max amount of {}'.format(MAX_MINT), 400

    currency_code = flask.request.args['currency_code']

    try:
        create_client()
        application.client.sendline("q as 000000000000000000000000000000dd")
        application.client.expect(r"sequence_number: ([0-9]+)", timeout=2)
        if application.client.match:
            next_dd_seq = int(application.client.match.groups()[0]) + 1
        else:
            raise Exception('DD sequence number not found')

        application.client.sendline(
            "a m {} {} {} use_base_units".format(auth_key, amount, currency_code))
        application.client.expect("Request submitted to faucet", timeout=10)
    except Exception:
        application.client.terminate(True)
        raise

    return str(next_dd_seq)


@application.route("/-/healthy", methods=('GET',))
def health_check():
    return "diem-faucet:ok"
