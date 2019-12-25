cmake_minimum_required(VERSION 3.2)
project (CuckooMinerPlugins)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/plugins)
set (CMAKE_CXX_FLAGS "--std=c++11")

set (OPT "-O3")
set (DOPT "-DPREFETCH")

set (FLAGS "-Wno-format -Wno-deprecated-declarations -D_POSIX_C_SOURCE=200112L ${OPT} ${DOPT} -I. ${CPPFLAGS} -pthread")
set (GPP_FLAGS "-march=native -m64 ${FLAGS}")
set (CFLAGS "-Wno-format -fomit-frame-pointer ${OPT}")
set (GCC_FLAGS "-m64 -std=gnu11 ${CFLAGS}")

set (CUDA_HOST_COMPILER_OVERRIDE $ENV{CUDA_HOST_COMPILER})
set (SKIP_CUCKATOO_GPU $ENV{SKIP_CUCKATOO_GPU})
set (SKIP_CUCKAROO_GPU $ENV{SKIP_CUCKAROO_GPU})
set (SKIP_CUCKAROOD_GPU $ENV{SKIP_CUCKAROOD_GPU})
set (SKIP_CUCKAROOM_CPU "1")
set (SKIP_CUCKAROOM_GPU $ENV{SKIP_CUCKAROOD_GPU})

#blake2b prerequisite
set (BLAKE_2B "cuckoo/src/crypto/blake2b-ref.c")

#common to all plugins
set (PLUGIN_BUILD_FLAGS "-DC_CALL_CONVENTION=1 -DSQUASH_OUTPUT=0")

#build CPU target
function (build_cpu_target sources target props)
	add_library(${target} SHARED ${sources})
	set_target_properties(${target} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} ${props} ${PLUGIN_BUILD_FLAGS}" PREFIX "" SUFFIX ".cuckooplugin")
endfunction()

function (build_cuda_target sources target props)
	if (BUILD_CUDA_PLUGINS)
		include("cmake/find_cuda.cmake")
		if (CUDA_FOUND)
			set (CUDA_PROPAGATE_HOST_FLAGS ON)
			cuda_add_library (${target} SHARED ${sources} OPTIONS "${props} ${PLUGIN_BUILD_FLAGS}")
			set_target_properties(${target} PROPERTIES PREFIX "" SUFFIX ".cuckooplugin")
		endif (CUDA_FOUND)
	endif (BUILD_CUDA_PLUGINS)
endfunction()

##################################################################################
### CUCKATOO (Asic Tuned) ########################################################
##################################################################################

### AT LEAN CPU TARGETS  #########################################

set (AT_LEAN_CPU_SRC
			cuckoo/src/cuckatoo/cuckatoo.h
			cuckoo/src/cuckatoo/bitmap.hpp
			cuckoo/src/cuckatoo/graph.hpp
			cuckoo/src/cuckatoo/compress.hpp
			cuckoo/src/threads/barrier.hpp
			cuckoo/src/crypto/siphash.hpp
			cuckoo/src/cuckatoo/lean.hpp
			cuckoo/src/cuckatoo/lean.cpp
			${BLAKE_2B})

build_cpu_target("${AT_LEAN_CPU_SRC}" cuckatoo_lean_cpu_compat_19 "-mno-avx2 -DNSIPHASH=4 -DATOMIC -DEDGEBITS=19")
build_cpu_target("${AT_LEAN_CPU_SRC}" cuckatoo_lean_cpu_compat_31 "-mno-avx2 -DNSIPHASH=4 -DATOMIC -DEDGEBITS=31")
build_cpu_target("${AT_LEAN_CPU_SRC}" cuckatoo_lean_cpu_avx2_31 "-mavx2 -DNSIPHASH=8 -DATOMIC -DEDGEBITS=31")

### AT MEAN CPU TARGETS  #########################################

set (AT_MEAN_CPU_SRC
			cuckoo/src/cuckatoo/cuckatoo.h
			cuckoo/src/cuckatoo/bitmap.hpp
			cuckoo/src/cuckatoo/graph.hpp
			cuckoo/src/cuckatoo/compress.hpp
			cuckoo/src/threads/barrier.hpp
			cuckoo/src/crypto/siphash.hpp
			cuckoo/src/cuckatoo/mean.hpp
			cuckoo/src/cuckatoo/mean.cpp
			${BLAKE_2B})

build_cpu_target("${AT_MEAN_CPU_SRC}" cuckatoo_mean_cpu_compat_19 "-mno-avx2 -DXBITS=2 -DNSIPHASH=4 -DSAVEEDGES -DEDGEBITS=19")
build_cpu_target("${AT_MEAN_CPU_SRC}" cuckatoo_mean_cpu_avx2_19 "-mavx2 -DXBITS=2 -DNSIPHASH=8 -DSAVEEDGES -DEDGEBITS=19")
build_cpu_target("${AT_MEAN_CPU_SRC}" cuckatoo_mean_cpu_compat_31 "-mno-avx2 -DXBITS=8 -DNSIPHASH=4 -DEXPANDROUND=8 -DCOMPRESSROUND=22 -DSAVEEDGES -DEDGEBITS=31")
build_cpu_target("${AT_MEAN_CPU_SRC}" cuckatoo_mean_cpu_avx2_31 "-mavx2 -DXBITS=8 -DNSIPHASH=8 -DEXPANDROUND=8 -DCOMPRESSROUND=22 -DSAVEEDGES -DEDGEBITS=31")

### AT LEAN CUDA TARGETS  #########################################

set (AT_LEAN_CUDA_SRC
			cuckoo/src/crypto/siphash.cuh
			cuckoo/src/cuckatoo/lean.cu
			${BLAKE_2B} )

if (NOT SKIP_CUCKATOO_GPU)
  build_cuda_target("${AT_LEAN_CUDA_SRC}" cuckatoo_lean_cuda_19 "-DEDGEBITS=19")
  build_cuda_target("${AT_LEAN_CUDA_SRC}" cuckatoo_lean_cuda_31 "-DEDGEBITS=31")
endif()

### AT MEAN CUDA TARGETS  #########################################

set (AT_MEAN_CUDA_SRC
			cuckoo/src/crypto/siphash.cuh
			cuckoo/src/cuckatoo/mean.cu
			${BLAKE_2B} )

if (NOT SKIP_CUCKATOO_GPU)
  build_cuda_target("${AT_MEAN_CUDA_SRC}" cuckatoo_mean_cuda_19 "-DNEPS_A=512 -DNEPS_B=384 -DIDXSHIFT=6 -DEDGEBITS=19")
  build_cuda_target("${AT_MEAN_CUDA_SRC}" cuckatoo_mean_cuda_gtx_31 "-DNEPS_A=135 -DNEPS_B=88 -DPART_BITS=1 -DFLUSHA=2 -DEDGEBITS=31")
  build_cuda_target("${AT_MEAN_CUDA_SRC}" cuckatoo_mean_cuda_rtx_31 "-DNEPS_A=135 -DNEPS_B=88 -DPART_BITS=0 -DEDGEBITS=31")
endif()

##################################################################################
### CUCKAROO (Asic Resistant)  ###################################################
##################################################################################

### AR CPU BUILDING  #########################################

set (AR_CPU_SRC
			cuckoo/src/cuckaroo/cuckaroo.hpp
			cuckoo/src/cuckaroo/bitmap.hpp
			cuckoo/src/cuckaroo/graph.hpp
			cuckoo/src/threads/barrier.hpp
			cuckoo/src/crypto/siphash.hpp
			cuckoo/src/cuckaroo/mean.hpp
			cuckoo/src/cuckaroo/mean.cpp
			${BLAKE_2B})

### AR CPU TARGETS #########################################

build_cpu_target("${AR_CPU_SRC}" cuckaroo_cpu_compat_19 "-mno-avx2 -DXBITS=2 -DNSIPHASH=4 -DEDGEBITS=19 -DSAVEEDGES")
build_cpu_target("${AR_CPU_SRC}" cuckaroo_cpu_avx2_19 "-mavx2 -DXBITS=2 -DNSIPHASH=8 -DEDGEBITS=19 -DSAVEEDGES")
build_cpu_target("${AR_CPU_SRC}" cuckaroo_cpu_compat_29 "-mno-avx2 -DNSIPHASH=4 -DEDGEBITS=29 -DSAVEEDGES")
build_cpu_target("${AR_CPU_SRC}" cuckaroo_cpu_avx2_29 "-mavx2 -DNSIPHASH=8 -DEDGEBITS=29 -DSAVEEDGES")

### AR CUDA TARGETS #########################################

set (AR_CUDA_SRC cuckoo/src/cuckaroo/mean.cu ${BLAKE_2B} )

if (NOT SKIP_CUCKAROO_GPU)
  build_cuda_target("${AR_CUDA_SRC}" cuckaroo_cuda_19 "-DEPS_A=4 -DEPS_B=3 -DIDXSHIFT=2 -DEDGEBITS=19")
  build_cuda_target("${AR_CUDA_SRC}" cuckaroo_cuda_29 "-DEDGEBITS=29")
endif()

##################################################################################
### CUCKAROOD (Asic Resistant)  ##################################################
##################################################################################

### AR CPU BUILDING  #########################################

set (AR2_CPU_SRC
			cuckoo/src/cuckarood/cuckarood.hpp
			cuckoo/src/cuckarood/bitmap.hpp
			cuckoo/src/cuckarood/graph.hpp
			cuckoo/src/threads/barrier.hpp
			cuckoo/src/crypto/siphash.hpp
			cuckoo/src/crypto/siphashxN.h
			cuckoo/src/cuckarood/mean.hpp
			cuckoo/src/cuckarood/mean.cpp
			${BLAKE_2B})

### AR CPU TARGETS #########################################

build_cpu_target("${AR2_CPU_SRC}" cuckarood_cpu_compat_19 "-mno-avx2 -DXBITS=2 -DNSIPHASH=4 -DEDGEBITS=19 -DSAVEEDGES")
build_cpu_target("${AR2_CPU_SRC}" cuckarood_cpu_avx2_19 "-mavx2 -DXBITS=2 -DNSIPHASH=8 -DEDGEBITS=19 -DSAVEEDGES")
build_cpu_target("${AR2_CPU_SRC}" cuckarood_cpu_compat_29 "-mno-avx2 -DNSIPHASH=4 -DEDGEBITS=29 -DSAVEEDGES")
build_cpu_target("${AR2_CPU_SRC}" cuckarood_cpu_avx2_29 "-mavx2 -DNSIPHASH=8 -DEDGEBITS=29 -DSAVEEDGES")

### AR CUDA TARGETS #########################################

set (AR2_CUDA_SRC cuckoo/src/cuckarood/kernel.cuh cuckoo/src/cuckarood/photon.cu ${BLAKE_2B} )

if (NOT SKIP_CUCKAROOD_GPU)
  build_cuda_target("${AR2_CUDA_SRC}" cuckarood_cuda_19 "-DXBITS=1 -DEPS_A=4 -DEPS_B=3 -DIDXSHIFT=8 -DEDGEBITS=19")
  build_cuda_target("${AR2_CUDA_SRC}" cuckarood_cuda_29 "-DEDGEBITS=29")
endif()

##################################################################################
### CUCKAROOM (Asic Resistant)  ##################################################
##################################################################################

### AR CPU BUILDING  #########################################

set (AR2_CPU_SRC
			cuckoo/src/cuckaroom/cuckaroom.hpp
			cuckoo/src/cuckaroom/bitmap.hpp
			cuckoo/src/cuckaroom/graph.hpp
			cuckoo/src/threads/barrier.hpp
			cuckoo/src/crypto/siphash.hpp
			cuckoo/src/crypto/siphashxN.h
			cuckoo/src/cuckaroom/mean.hpp
			cuckoo/src/cuckaroom/mean.cpp
			${BLAKE_2B})

### AR CPU TARGETS #########################################

if (NOT SKIP_CUCKAROOM_CPU)
  build_cpu_target("${AR2_CPU_SRC}" cuckaroom_cpu_compat_19 "-mno-avx2 -DXBITS=2 -DNSIPHASH=4 -DEDGEBITS=19 -DSAVEEDGES")
  build_cpu_target("${AR2_CPU_SRC}" cuckaroom_cpu_avx2_19 "-mavx2 -DXBITS=2 -DNSIPHASH=8 -DEDGEBITS=19 -DSAVEEDGES")
  build_cpu_target("${AR2_CPU_SRC}" cuckaroom_cpu_compat_29 "-mno-avx2 -DNSIPHASH=4 -DEDGEBITS=29 -DSAVEEDGES")
  build_cpu_target("${AR2_CPU_SRC}" cuckaroom_cpu_avx2_29 "-mavx2 -DNSIPHASH=8 -DEDGEBITS=29 -DSAVEEDGES")
endif()

### AR CUDA TARGETS #########################################

set (AR2_CUDA_SRC cuckoo/src/cuckaroom/kernel.cuh cuckoo/src/cuckaroom/mean.cu ${BLAKE_2B} )

if (NOT SKIP_CUCKAROOM_GPU)
  build_cuda_target("${AR2_CUDA_SRC}" cuckaroom_cuda_19 "-DXBITS=1 -DEPS_A=4 -DEPS_B=3 -DIDXSHIFT=8 -DEDGEBITS=19")
  build_cuda_target("${AR2_CUDA_SRC}" cuckaroom_cuda_29 "-DEDGEBITS=29")
endif()
