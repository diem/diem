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


MAX_MINT = 10 ** 19  # 10 trillion libras


def create_client():
    if application.client is None or not application.client.isalive():
        # If we have comma separated list take a random one
        ac_hosts = os.environ['AC_HOST'].split(',')
        ac_host = random.choice(ac_hosts)
        ac_port = os.environ['AC_PORT']

        print("Connecting to ac on: {}:{}".format(ac_host, ac_port))
        cmd = "/opt/libra/bin/cli --host {} --port {} -m {}".format(
            ac_host,
            ac_port,
            "/opt/libra/etc/mint.key")

        application.client = pexpect.spawn(cmd)
        application.client.delaybeforesend = 0.1
        application.client.expect("Please, input commands")


application = flask.Flask(__name__)
application.client = None
print(sys.version, platform.python_version())
create_client()


@application.route("/", methods=('POST',))
def send_transaction():
    address = flask.request.args['address']

    # Return immediately if address is invalid
    if re.match('^[a-f0-9]{64}$', address) is None:
        return 'Malformed address', 400

    try:
        amount = decimal.Decimal(flask.request.args['amount'])
    except decimal.InvalidOperation:
        return 'Bad amount', 400

    if amount > MAX_MINT:
        return 'Exceeded max amount of {}'.format(MAX_MINT / (10 ** 6)), 400

    try:
        create_client()
        application.client.sendline(
            "a m {} {}".format(address, amount / (10 ** 6)))
        application.client.expect("Mint request submitted", timeout=2)

        application.client.sendline("a la")
        application.client.expect(r"sequence_number: ([0-9]+)", timeout=1)
        application.client.terminate(True)
    except pexpect.exceptions.ExceptionPexpect:
        application.client.terminate(True)
        raise

    return application.client.match.groups()[0]
