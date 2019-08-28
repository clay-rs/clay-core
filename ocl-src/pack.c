__kernel void pack(
    int2 size,
    __global uchar *image
    __global const float *buffer,
) {
    int2 pos = (int2)(get_global_id(0), get_global_id(1));
    int idx = pos.x + pos.y*size.x;

    float3 color = vload3(idx, buffer);
    uchar3 pixel = convert_uchar3(255.0f*clamp(color, 0.0f, 1.0f));
    vstore3(pixel, idx, image);
}
