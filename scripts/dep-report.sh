#!/bin/bash

first_cargo_lock_rev=508429c63c0a40bf4b2726ff9c1b60a80f0a50d0
from_rev=${1:-${first_cargo_lock_rev}}

git fetch --quiet --all
REVS=$(git rev-list --reverse upstream/master ^${from_rev})

if [[ "${from_rev}" == "${first_cargo_lock_rev}" ]]; then
    echo date,rev,total,direct,lsr_total,lsr_direct,lec_total,lec_direct,dups
fi

for rev in ${REVS}; do
    date=$(git show --no-patch --format=%cd --date='format:%Y%m%d %H:%M:%S %z' ${rev})
    git checkout --quiet --force ${rev}
    total=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind thirdparty --omit-edges-into libra-workspace-hack | wc -l)
    direct=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind directthirdparty --omit-edges-into libra-workspace-hack | wc -l)
    lsr_total=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind thirdparty --omit-edges-into libra-workspace-hack -p safety-rules | wc -l)
    lsr_direct=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind directthirdparty --omit-edges-into libra-workspace-hack -p safety-rules | wc -l)
    lec_total=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind thirdparty --omit-edges-into libra-workspace-hack -p executor | wc -l)
    lec_direct=$(cargo +stable guppy resolve-cargo --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind directthirdparty --omit-edges-into libra-workspace-hack -p executor | wc -l)
    dups=$(cargo +stable guppy dups --target x86_64-unknown-linux-gnu | wc -l)
    echo ${date},${rev},${total},${direct},${lsr_total},${lsr_direct},${lec_total},${lec_direct},${dups}
done
