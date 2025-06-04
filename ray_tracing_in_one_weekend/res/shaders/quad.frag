#version 460 core

layout (binding = 0) uniform sampler2D screen;
layout (binding = 1) uniform sampler2D frame_counter;

out vec4 out_color;

#define KERNEL_SIZE 3
const float GAUSSIAN_KERNEL[KERNEL_SIZE * KERNEL_SIZE] = float[](0.0449, 0.1221, 0.0449, 0.1221, 0.3319, 0.1221, 0.0449, 0.1221, 0.0449);

void main() {
    vec2 resolution = vec2(textureSize(screen, 0));
    vec2 pixel_coords = vec2(gl_FragCoord.xy);

    out_color = vec4(0.0);
    int half_size = KERNEL_SIZE / 2;
    for (int i = -half_size; i <= half_size; i++) {
        for (int j = -half_size; j <= half_size; j++) {
            vec2 offset = vec2(i, j);
            vec2 pixel = pixel_coords + offset * 0.25;
            int idx = (i + half_size) * 3 + (j + half_size);

            float kernel_weight = GAUSSIAN_KERNEL[idx];
            vec4 color = texture2D(screen, pixel / resolution);
            out_color += color * kernel_weight;
        }
    }

    //    out_color = texture2D(screen, pixel_coords / resolution);
}