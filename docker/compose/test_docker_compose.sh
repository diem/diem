#!/usr/bin/expect
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

set timeout 5

set basedir [file normalize [file dirname $argv0]]
cd $basedir

### Test the validator/faucet
spawn /bin/bash
cd $basedir
expect_before {
    timeout { puts "\rERROR: Timeout!\r"; exit 1 }
    eof { puts "\rERROR: eof!\r"; exit 1 }
}

send "cd validator-testnet\r"
send "docker-compose up --remove-orphans\r"
# Order is non-deterministic, so we test both ways
expect {
  "validator_1*Diem is running" { expect  "faucet_1*running" }
  "faucet_1*running*" { expect "validator_1*Diem is running" }
}

sleep 3

### Test the CLI client
spawn /bin/bash
cd $basedir
expect_before {
    timeout { puts "\rERROR: Timeout!\r"; exit 1 }
    eof { puts "\rERROR: eof!\r"; exit 1 }
}

send "cd client-cli\r"
send "docker-compose run client-cli\r"
expect "diem%"
send "a c\r"
expect "Created/retrieved local account"
send "a m 0 10 XUS\r"
expect "Finished sending coins from faucet!"
send "q b 0\r"
expect "Balance is: 10.0*XUS"


puts "\rPASSED!\r"
