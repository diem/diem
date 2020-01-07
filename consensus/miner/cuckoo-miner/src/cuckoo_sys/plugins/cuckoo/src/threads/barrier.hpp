#pragma once
#include <pthread.h>
#include <errno.h>

#ifdef __APPLE__
typedef int pthread_barrierattr_t;
#endif

class trim_barrier {
  pthread_mutex_t mutex;
  pthread_cond_t cond;
  unsigned limit;
  unsigned count;
  int phase;

public:
  trim_barrier(unsigned int count) {
    pthread_mutex_init(&mutex, 0);
    pthread_cond_init(&cond, 0);
    limit = count;
  }
  
  ~trim_barrier() {
    pthread_mutex_destroy(&mutex);
    pthread_cond_destroy(&cond);
  }
  
  void clear() {
    count = phase = 0;
  }

  void abort() {
    pthread_mutex_lock(&mutex);
    phase = -1;
    pthread_mutex_unlock(&mutex);
  }
  
  bool aborted() {
    return phase < 0;
  }

  void wait() {
    pthread_mutex_lock(&mutex);
    int wait_phase = phase;
    if (++count >= limit) {
      if (wait_phase >= 0) {
        phase = wait_phase + 1;
        count = 0;
      }
      pthread_cond_broadcast(&cond);
    } else if (wait_phase >= 0) {
      do
        pthread_cond_wait(&cond, &mutex);
      while (phase == wait_phase);
    }
    pthread_mutex_unlock(&mutex);
    if (wait_phase < 0)
      pthread_exit(NULL);
  }
};
