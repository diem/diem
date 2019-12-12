#!/bin/sh

### Find script directory and load helper functions.
scriptdir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
. ${scriptdir}/shared_test_functions.sh


### Project-specific constants and setup

out="${bindir}/tests-output"
out_valgrind="${bindir}/tests-valgrind"

# Constants used in multiple scenarios for this project.
password="hunter2"

# Find system scrypt, and ensure it supports -P.
system_scrypt=$( find_system scrypt enc -P )


### Run tests using project-specific constants
run_scenarios ${scriptdir}/??-*.sh
