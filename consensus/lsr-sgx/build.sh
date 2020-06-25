set -e

ENCLAVE_NAME=lsr-sgx

# Build target path
TARGET_PATH=../../target/x86_64-fortanix-unknown-sgx/debug/$ENCLAVE_NAME


cargo +nightly build --target=x86_64-fortanix-unknown-sgx


ftxsgx-elf2sgxs $TARGET_PATH --heap-size 0x20000 --stack-size 0x20000 --threads 6 --debug

