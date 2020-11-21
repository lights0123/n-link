#pragma once
#include "stdint.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef int64_t time_t;
typedef int32_t suseconds_t;

struct timeval {
    time_t      tv_sec;     /* seconds */
    suseconds_t tv_usec;    /* microseconds */
};

int gettimeofday(struct timeval *tv, void *tz);
#ifdef __cplusplus
}
#endif
