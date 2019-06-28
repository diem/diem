#!/bin/sh
set -ex

IMAGE="${1:-853397791086.dkr.ecr.us-west-2.amazonaws.com/socket_muxer_bench_client:latest}"

BENCH_PATTERN="${2:-remote_tcp}"

docker run \
    --env TCP_ADDR="$TCP_ADDR" \
    --env TCP_NOISE_ADDR="$TCP_NOISE_ADDR" \
    --env TCP_MUXER_ADDR="$TCP_MUXER_ADDR" \
    --env TCP_NOISE_MUXER_ADDR="$TCP_NOISE_MUXER_ADDR" \
    --env BENCH_PATTERN="$BENCH_PATTERN" \
    "$IMAGE"
