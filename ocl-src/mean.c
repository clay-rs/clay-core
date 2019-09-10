__kernel void mean(
    int2 size,
    int dst_passes,
    int src_passes,
    __global float *dst_buffer,
    __global const float *src_buffer
) {
    int2 pos = (int2)(get_global_id(0), get_global_id(1));
    int idx = pos.x + pos.y*size.x;

    float3 dst_color = vload3(idx, dst_buffer);
    float3 src_color = vload3(idx, src_buffer);
    dst_color = (dst_color*dst_passes + src_color)/(dst_passes + src_passes);
    vstore3(dst_color, idx, dst_buffer);
}
