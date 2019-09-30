#pragma once


#define FILTER_ARGS_DEF \
    int2 size,
    __global float *dst_buffer,
    __global const float *src_buffer

#define FILTER_ARGS \
    size, dst_buffer, src_buffer
