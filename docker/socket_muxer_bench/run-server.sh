#!/bin/sh
set -ex

IMAGE="${1:-853397791086.dkr.ecr.us-west-2.amazonaws.com/socket_muxer_bench_server:latest}"

docker run \
    --publish 52720:52720 \
    --publish 52721:52721 \
    --publish 52722:52722 \
    --publish 52723:52723 \
    "$IMAGE"
