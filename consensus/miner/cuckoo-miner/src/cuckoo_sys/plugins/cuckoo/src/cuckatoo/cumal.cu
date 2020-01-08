#include <stdio.h>
#include <inttypes.h>
#include <assert.h>

int main(int argc, char **argv) {
  size_t bufferMB;
  void *buffer;
  int device = argc > 1 ? atoi(argv[argc-1]) : 1;
  int nDevices;
  cudaGetDeviceCount(&nDevices);
  assert(device < nDevices);
  cudaDeviceProp prop;
  cudaGetDeviceProperties(&prop, device);
  uint64_t dbytes = prop.totalGlobalMem;
  int availMB = dbytes >> 20;
  printf("%s with %d MB @ %d bits x %dMHz\n", prop.name, availMB, prop.memoryBusWidth, prop.memoryClockRate/1000);

  cudaSetDevice(device);
  for (bufferMB = availMB; ; bufferMB -= 1) {
    int ret = cudaMalloc((void**)&buffer, bufferMB << 20);
    if (ret) printf("cudaMalloc(%d MB) returned %d\n", bufferMB, ret);
    else break;
  }
  printf("cudaMalloc(%d MB) succeeded %d\n", bufferMB);
  cudaFree(buffer);

  return 0;
}
