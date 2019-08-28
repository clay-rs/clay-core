#pragma once

// placeholder args
#define IDENTITY_FILTER_ARGS_DEF int
#define IDENTITY_FILTER_ARGS 0


float3 identity_filter_apply(
    int2 pos, int2 size,
    __global const float *buffer,
    IDENTITY_FILTER_ARGS_DEF
) {
    return vload3(idx, buffer);
}
