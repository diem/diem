# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

#!/bin/sh
flist=`ls *.dot`
for entry in $flist
do
    fname="${entry%.*}"
    echo "$entry ==> $fname.svg"
    dot -Tsvg $fname.dot -o $fname.svg
done
