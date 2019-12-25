.POSIX:
.SUFFIXES:

OPT ?= -O3

GCC_ARCH_FLAGS ?= -march=native
GPP_ARCH_FLAGS ?= -march=native

# -Wno-deprecated-declarations shuts up Apple OSX clang
FLAGS ?= -Wall -Wno-format -Wno-deprecated-declarations -D_POSIX_C_SOURCE=200112L $(OPT) -DPREFETCH -I. $(CPPFLAGS) -pthread
GPP ?= g++ $(GPP_ARCH_FLAGS) -std=c++11 $(FLAGS)
CFLAGS ?= -Wall -Wno-format -fomit-frame-pointer $(OPT)
GCC ?= gcc $(GCC_ARCH_FLAGS) -std=gnu11 $(CFLAGS)
BLAKE_2B_SRC ?= ../crypto/blake2b-ref.c
NVCC ?= nvcc -std=c++11 

all : simpletest meantest

simpletest:     simple19
	./simple19 -n 71

meantest:	mean29x4
	./mean29x4 -n 671 -t 4 -s

simple19:	../crypto/siphash.hpp cuckaroo.hpp  bitmap.hpp graph.hpp simple.cpp Makefile
	$(GPP) -o $@ -DPROOFSIZE=42 -DEDGEBITS=19 simple.cpp $(BLAKE_2B_SRC)

simple29:	../crypto/siphash.hpp cuckaroo.hpp  bitmap.hpp graph.hpp simple.cpp Makefile
	$(GPP) -o $@ -DPROOFSIZE=42 -DEDGEBITS=29 simple.cpp $(BLAKE_2B_SRC)

mean19x1:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DXBITS=2 -DNSIPHASH=1 -DEDGEBITS=19 mean.cpp $(BLAKE_2B_SRC)

mean19x8:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DXBITS=2 -DNSIPHASH=8 -DEDGEBITS=19 mean.cpp $(BLAKE_2B_SRC)

mean29x4:	cuckaroo.hpp  bitmap.hpp graph.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mno-avx2 -DNSIPHASH=4 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean29x8:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean29x8s:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DSAVEEDGES -DNSIPHASH=8 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean29x1:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean30x1:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DEXPANDROUND=10 -DCOMPRESSROUND=22 -DEDGEBITS=30 mean.cpp $(BLAKE_2B_SRC)

mean30x8:	cuckaroo.hpp  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DEXPANDROUND=10 -DCOMPRESSROUND=22 -DEDGEBITS=30 mean.cpp $(BLAKE_2B_SRC)

cuda19:		../crypto/siphash.cuh compress.hpp graph.hpp mean.cu Makefile
	$(NVCC) -o $@ -DEPS_A=4 -DEPS_B=3 -DIDXSHIFT=2 -DEDGEBITS=19 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda29:		../crypto/siphash.cuh compress.hpp graph.hpp mean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=29 -arch sm_35 mean.cu $(BLAKE_2B_SRC)
