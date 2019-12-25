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

all : simpletest leantest meantest

simpletest:     simple19
	./simple19 -n 68

leantest:       lean19x1
	./lean19x1 -n 68

meantest:	mean29x4
	./mean29x4 -n 20 -t 4 -s

verifytest:     lean19x1 verify19
	./lean19x1 -n 68 | grep ^Sol | ./verify19 -n 68

simple19:	../crypto/siphash.hpp cuckatoo.h  bitmap.hpp graph.hpp simple.cpp Makefile
	$(GPP) -o $@ -DPROOFSIZE=42 -DEDGEBITS=19 simple.cpp $(BLAKE_2B_SRC)

verify19:       ../crypto/siphash.hpp cuckatoo.h cuckatoo.c simple.cpp Makefile
	$(GPP) -o $@ -DPROOFSIZE=42 -DEDGEBITS=19 cuckatoo.c $(BLAKE_2B_SRC)

simple29:	../crypto/siphash.hpp cuckatoo.h  bitmap.hpp graph.hpp simple.cpp Makefile
	$(GPP) -o $@ -DPROOFSIZE=42 -DEDGEBITS=29 simple.cpp $(BLAKE_2B_SRC)

lean19x1:		../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DATOMIC -DEDGEBITS=19 lean.cpp $(BLAKE_2B_SRC)

lean29x8:	../threads/barrier.hpp ../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DATOMIC -DEDGEBITS=29 lean.cpp $(BLAKE_2B_SRC)

lean31x1:	../threads/barrier.hpp ../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DATOMIC -DEDGEBITS=31 lean.cpp $(BLAKE_2B_SRC)

lean31x4:	../threads/barrier.hpp ../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -mno-avx2 -DNSIPHASH=4 -DATOMIC -DEDGEBITS=31 lean.cpp $(BLAKE_2B_SRC)

lean31x8:	../threads/barrier.hpp ../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DATOMIC -DEDGEBITS=31 lean.cpp $(BLAKE_2B_SRC)

lean32x8:	../threads/barrier.hpp ../crypto/siphash.hpp cuckatoo.h  bitmap.hpp compress.hpp graph.hpp lean.hpp lean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DATOMIC -DEDGEBITS=32 lean.cpp $(BLAKE_2B_SRC)

mean19x1:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DXBITS=2 -DNSIPHASH=1 -DEDGEBITS=19 mean.cpp $(BLAKE_2B_SRC)

mean19x8:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DXBITS=2 -DNSIPHASH=8 -DEDGEBITS=19 mean.cpp $(BLAKE_2B_SRC)

mean29x4:	cuckatoo.h  bitmap.hpp graph.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mno-avx2 -DNSIPHASH=4 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean29x8:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean30x8:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DEXPANDROUND=10 -DCOMPRESSROUND=22 -DEDGEBITS=30 mean.cpp $(BLAKE_2B_SRC)

mean31x1:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DXBITS=8 -DEXPANDROUND=8 -DCOMPRESSROUND=22 -DEDGEBITS=31 mean.cpp $(BLAKE_2B_SRC)

mean31x4:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mno-avx2 -DNSIPHASH=4 -DXBITS=8 -DEXPANDROUND=8 -DCOMPRESSROUND=22 -DEDGEBITS=31 mean.cpp $(BLAKE_2B_SRC)

mean31x8:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DNSIPHASH=8 -DXBITS=8 -DEXPANDROUND=8 -DCOMPRESSROUND=22 -DEDGEBITS=31 mean.cpp $(BLAKE_2B_SRC)

mean29x8s:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -mavx2 -DSAVEEDGES -DNSIPHASH=8 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean29x1:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DEDGEBITS=29 mean.cpp $(BLAKE_2B_SRC)

mean30x1:	cuckatoo.h  bitmap.hpp graph.hpp ../threads/barrier.hpp ../crypto/siphash.hpp mean.hpp mean.cpp Makefile
	$(GPP) -o $@ -DNSIPHASH=1 -DEXPANDROUND=10 -DCOMPRESSROUND=22 -DEDGEBITS=30 mean.cpp $(BLAKE_2B_SRC)

lcuda19:	../crypto/siphash.cuh lean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=19 -arch sm_35 lean.cu $(BLAKE_2B_SRC)

lcuda29:	../crypto/siphash.cuh lean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=29 -arch sm_35 lean.cu $(BLAKE_2B_SRC)

lcuda30:	../crypto/siphash.cuh lean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=30 -arch sm_35 lean.cu $(BLAKE_2B_SRC)

lcuda31:	../crypto/siphash.cuh lean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=31 -arch sm_35 lean.cu $(BLAKE_2B_SRC)

cuda19:		../crypto/siphash.cuh mean.cu Makefile
	$(NVCC) -o $@ -DNEPS_A=512 -DNEPS_B=384 -DIDXSHIFT=6 -DEDGEBITS=19 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda29:		../crypto/siphash.cuh mean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=29 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda30:		../crypto/siphash.cuh mean.cu Makefile
	$(NVCC) -o $@ -DEDGEBITS=30 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda31.1:	../crypto/siphash.cuh mean.cu graph.hpp Makefile
	$(NVCC) -o $@ -DFLUSHA=2 -DNEPS_A=135 -DNEPS_B=88 -DPART_BITS=1 -DEDGEBITS=31 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda31.0:	../crypto/siphash.cuh mean.cu graph.hpp Makefile
	$(NVCC) -o $@ -DNEPS_A=133 -DNEPS_B=85 -DPART_BITS=0 -DEDGEBITS=31 -arch sm_75 mean.cu $(BLAKE_2B_SRC)

cuda32.2:	../crypto/siphash.cuh mean.cu Makefile
	$(NVCC) -o $@ -DNEPS_A=135 -DNEPS_B=88 -DPART_BITS=2 -DEDGEBITS=32 -arch sm_35 mean.cu $(BLAKE_2B_SRC)

cuda32.1:	../crypto/siphash.cuh mean.cu Makefile
	$(NVCC) -o $@ -DNEPS_A=133 -DNEPS_B=85 -DPART_BITS=1 -DEDGEBITS=32 -arch sm_75 mean.cu $(BLAKE_2B_SRC)

cumal:		cumal.cu
	$(NVCC) -o $@ cumal.cu
