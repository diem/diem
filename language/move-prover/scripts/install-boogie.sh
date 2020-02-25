#!/bin/bash -e

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

dotnet tool uninstall --global Boogie
dotnet tool install --global Boogie
