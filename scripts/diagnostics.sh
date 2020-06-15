#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

INPUT=
OUTPUT=
NODES=''
BROKEN_NODES=''

function usage {
  echo "Usage:"
  echo "diagnostics.sh -n ./nodes.csv -o /tmp/outputfile"
  echo "the csv should be of the form <operator>,<if disabled "true" or empty>,<[dns|ip][:port|]>"
  echo "if no port is provided, port 80 is assumed."
}

while getopts "n:o:h" arg; do
  case $arg in
    n)
      INPUT=$OPTARG
      ;;
    o)
      OUTPUT=$OPTARG
      ;;
    h)
      usage;
      exit 0;
      ;;
  esac
done

[ ! -f $INPUT ] && { echo "$INPUT file not found"; usage; exit 99; }
[ "$OUTPUT" != "" ] && touch $OUTPUT || { echo "$OUTPUT file not writable"; usage; exit 99; }

while read operator broken ip
do
    if [[ ! -z "$ip" ]] && [[ "$broken" != "true" ]]; then
      #if the ip doesn't have a port number attached, assume :80
      if [[ $ip !=  *":"* ]]; then
        ip=$ip:80
      fi

      diag=$( cargo run -p cluster-test -- --swarm --diag --mint-file /tmp/config/mint.key --peers $ip )
      operational=$?

      if [[ "$operational" == "0" ]]; then
        if [[ ! -z "$NODES" ]]; then
          NODES=${NODES},
        fi
        NODES=${NODES}$ip
      else
        echo Failed Operator: $operator >> ${OUTPUT}
        echo $diag >> ${OUTPUT}
        echo >> ${OUTPUT}
      fi
    fi
done < $INPUT

coutput=$( cargo run -p cluster-test -- --swarm --emit-tx --mint-file /tmp/config/mint.key --peers $NODES)
echo $coutput >> $OUTPUT

result=0
if [ -f "$OUTPUT" ]; then
  result=1
fi

echo OutputFile:
cat $OUTPUT

exit $result
