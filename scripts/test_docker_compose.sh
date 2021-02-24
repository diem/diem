#!/usr/bin/expect
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

set timeout 5

cd [file dirname $argv0]


### Test the validator/faucet
spawn $env(SHELL)
cd [file dirname $argv0]
expect_before {
    timeout { puts "\rERROR: Timeout!\r"; exit 1 }
    eof { puts "\rERROR: eof!\r"; exit 1 }
}

send "../docker/compose/validator-testnet\r"
send "docker-compose up --remove-orphans\r"
# Order is non-deterministic, so we test both ways
expect {
  "*validator_1*Diem is running*" { expect  "*faucet_1*running*" }
  "*faucet_1*running*" { expect "*validator_1*Diem is running*" }
}


### Test the CLI client
spawn $env(SHELL)
cd [file dirname $argv0]
expect_before {
    timeout { puts "\rERROR: Timeout!\r"; exit 1 }
    eof { puts "\rERROR: eof!\r"; exit 1 }
}

send "cd ../docker/compose/client\r"
send "docker-compose run client\r"
expect "diem%"
send "a c\r"
expect "Created/retrieved local account"
send "a m 0 10 XUS\r"
expect "Finished sending coins from faucet!"
send "q b 0\r"
expect "Balance is: 10.0*XUS"


puts "\rPASSED!\r"
