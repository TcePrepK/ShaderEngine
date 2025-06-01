#version 460 core

uniform vec2 resolution;
uniform float time;

out vec4 out_color;

#define SAMPLE_PER_PIXEL 16
#define MAX_SPHERES 4
#define MAX_BOUNCES 20
#define EPSILON 0.0001

// No includes in these files
#include "utils/math.glsl"
#include "utils/sdf_funcs.glsl"
#include "utils/ray_utils.glsl"
// Complex includes involved here
#include "hit_record.glsl"
#include "material.glsl"
#include "sphere.glsl"

const Sphere all_spheres[MAX_SPHERES] = Sphere[](
    Sphere(vec3(0.0, -0.01, 0.0), 0.5, Material(1, vec3(1.0, 0.1, 0.1), 0.0, 0.0)),
    Sphere(vec3(1.2, -0.01, 0.0), 0.5, Material(0, vec3(0.1, 1.0, 0.1), 0.0, 0.0)),
    Sphere(vec3(-1.2, -0.01, 0.0), 0.5, Material(0, vec3(0.1, 0.1, 1.0), 0.0, 0.0)),
    Sphere(vec3(0, -500.5, 0.0), 500.0, Material(1, vec3(0.2), 0.0, 0.0))
);

HitRecord check_world(in Ray ray) {
    float closest_so_far = 1000.0;
    HitRecord closest_hit_record = NO_HIT;
    for (int i = 0; i < MAX_SPHERES; i++) {
        HitRecord hit_record = sphere_hit(all_spheres[i], ray);
        if (hit_record.time > 0.0) {
            if (hit_record.time < closest_so_far) {
                closest_so_far = hit_record.time;
                closest_hit_record = hit_record;
            }
        }
    }

    return closest_hit_record;
}

vec3 get_color(in Ray ray) {
    vec3 color = vec3(1.0);

    int total_bounces = 0;
    Ray current_ray = ray;
    for (int i = 0; i < MAX_BOUNCES; i++) {
        HitRecord closest_hit = check_world(current_ray);
        total_bounces++;

        if (closest_hit == NO_HIT) {
            color *= sky_color(ray);
            break;
        }

        float hit_time = closest_hit.time;
        if (hit_time > 0.0) {
            Ray scattered_ray;
            vec3 attenuation;

            Material material = closest_hit.material;
            int material_type = material.material_type;
            bool scattered;
            if (material_type == 0) {
                scattered = scatter_lambertian(current_ray, closest_hit, material, attenuation, scattered_ray);
            } else if (material_type == 1) {
                scattered = scatter_metal(current_ray, closest_hit, material, attenuation, scattered_ray);
            }

            if (scattered) {
                color *= attenuation;
                current_ray = scattered_ray;
            }
        }
    }

    return color;
}

void main() {
    vec2 pixel = gl_FragCoord.xy / resolution;
    pixel.y = 1.0 - pixel.y;

    vec3 color = vec3(0.0);
    for (int i = 0; i < SAMPLE_PER_PIXEL; i++) {
        vec2 random_offset = random_vec2(pixel + i) * 0.5 / resolution;
        vec2 offset = pixel + random_offset;
        Ray ray = camera_ray(offset, resolution, time);
        color += get_color(ray);
    }

    vec3 final_color = color / SAMPLE_PER_PIXEL;
    final_color = pow(final_color, vec3(1.0 / 2.2));
    out_color = vec4(final_color, 1.0);
}