"""
Dummy web server that proxies incoming requests to local client that owns association keys  

Installation:
	virtualenv -p python3 ~/env
	source ~/env/bin/activate 
	pip install flask gunicorn pexpect

To run:
	gunicorn --bind 0.0.0.0:8000 server

"""
import flask

import decimal
import os
import pexpect
import platform
import random
import re
import sys

print(sys.version)
print(platform.python_version())


def setup_app():
    application = flask.Flask(__name__)
    ac_host = os.environ['AC_HOST']
    ac_port = os.environ['AC_PORT']

    # If we have comma separated list take a random one
    ac_hosts = ac_host.split(',')
    ac_host = random.choice(ac_hosts)

    print("Connecting to ac on: {}:{}".format(ac_host, ac_port))

    cmd = "/opt/libra/bin/client --host {} --port {} -m /opt/libra/etc/mint.key --validator_set_file /opt/libra/etc/trusted_peers.config.toml".format(
        ac_host, ac_port)
    application.client = pexpect.spawn(cmd)
    application.client.expect("Please, input commands")
    return application


application = setup_app()

MAX_MINT = 10 ** 12

@application.route("/")
def send_transaction():
    address = flask.request.args['address']

    # Return immediately if address is invalid
    if re.match('^[a-f0-9]{64}$', address) is None:
        return 'Malformed address', 400

    try:
        amount = decimal.Decimal(flask.request.args['amount'])
    except:
        return 'Bad amount', 400

    if amount > MAX_MINT:
        return 'Exceeded max mint amount of {}'.format(MAX_MINT / (10 ** 6)), 400

    application.client.sendline("a m {} {}".format(address, amount / (10 ** 6)))
    application.client.expect("Mint request submitted", timeout=2)

    application.client.sendline("a la")
    application.client.expect(r"sequence_number: ([0-9]+)", timeout=1)
    return application.client.match.groups()[0]
