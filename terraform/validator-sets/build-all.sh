#!/bin/sh
set -e

./build.sh dev -n 4
./build.sh 100 -n 100
