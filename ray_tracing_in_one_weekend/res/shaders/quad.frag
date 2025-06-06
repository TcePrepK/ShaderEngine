#version 460 core

layout (binding = 0) uniform sampler2D screen;
layout (binding = 1) uniform sampler2D frame_counter;

out vec4 out_color;

#define KERNEL_SIZE 3
//const float GAUSSIAN_KERNEL[KERNEL_SIZE * KERNEL_SIZE] = float[](0.0449, 0.1221, 0.0449, 0.1221, 0.3319, 0.1221, 0.0449, 0.1221, 0.0449);
const float GAUSSIAN_KERNEL[9] = float[](1.0, 4.0, 1.0, 4.0, 0.0, 4.0, 1.0, 4.0, 1.0);
const float KERNEL_SCALE = 1.0 / 20.0;

float gaussian(in float v, in float sigma) {
    return exp(-(v * v) / (2.0 * sigma * sigma));
}

vec3 gaussian(in vec3 v, in float sigma) {
    return exp(-(v * v) / (2.0 * sigma * sigma));
}

void main() {
    vec2 resolution = vec2(textureSize(screen, 0));
    vec2 pixel_coords = vec2(gl_FragCoord.xy);

    //    out_color = vec4(0.0);
    //    int half_size = KERNEL_SIZE / 2;
    //    for (int i = -half_size; i <= half_size; i++) {
    //        for (int j = -half_size; j <= half_size; j++) {
    //            vec2 offset = vec2(i, j);
    //            vec2 pixel = pixel_coords + offset * 0.5;
    //            int idx = (i + half_size) * 3 + (j + half_size);
    //
    //            float kernel_weight = GAUSSIAN_KERNEL[idx];
    //            vec4 color = texture2D(screen, pixel / resolution);
    //            out_color += color * kernel_weight;
    //        }
    //    }

    float sigma_fr = 0.5;
    float sigma_gs = 0.5;

    vec3 total_color = vec3(0.0);
    vec3 total_weight = vec3(0.0);

    vec4 main_color = texture2D(screen, pixel_coords / resolution);
    for (int i = 0; i < KERNEL_SIZE; i++) {
        for (int j = 0; j < KERNEL_SIZE; j++) {
            vec2 offset = vec2(i, j) - KERNEL_SIZE / 2;
            int idx = i * KERNEL_SIZE + j;
            vec2 pixel = pixel_coords + offset;

            vec4 neighbor_color = texture2D(screen, pixel / resolution);
            vec3 color_diff = abs(main_color - neighbor_color).rgb;

            vec3 gauss_fr = gaussian(color_diff, sigma_fr);
            float gauss_gs = gaussian(length(offset), sigma_gs);
            vec3 gauss_fr_gs = gauss_fr * gauss_gs;

            total_color += neighbor_color.rgb * gauss_fr_gs;
            total_weight += gauss_fr_gs;
        }
    }

    //    out_color = vec4(total_weight / 25.0, 1.0);
    out_color = vec4(total_color / total_weight, 1.0);
    //    out_color = main_color;

    //    out_color = texture2D(frame_counter, pixel_coords / resolution);
}