set -e

ENCLAVE=lsr-sgx
SAFETY_RULES=safety-rules

# first build the LSR, which invokes lsr-sgx...
cargo build -p $SAFETY_RULES

# build enclave
cd $ENCLAVE
./build.sh
cd -

# test
cargo x test -p $SAFETY_RULES -- --nocapture
