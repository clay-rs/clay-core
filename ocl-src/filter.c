#include <__gen/filter.h>


__kernel void filter(
    int2 size,
    __global float *dst_buffer,
    __global const float *src_buffer,
    __FILTER_ARGS_DEF
) {
    int2 pos = (int2)(get_global_id(0), get_global_id(1));
    int idx = pos.x + pos.y*size.x;

    float3 dst_color = __filter_apply(pos, size, src_buffer, __FILTER_ARGS);
    vstore3(dst_color, idx, dst_buffer);
}
