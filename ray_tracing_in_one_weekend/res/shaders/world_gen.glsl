/* <ignore> */
// No includes in these files
#include "utils/math.glsl"
#include "utils/sdf_funcs.glsl"
#include "utils/ray_utils.glsl"
// Complex includes involved here
#include "ray_tracing/hit_record.glsl"
#include "ray_tracing/material.glsl"
#include "ray_tracing/sphere.glsl"
/* </ignore> */

Material random_material(in vec3 seed) {
    int material_type = int(random(seed) * 4.0);

    vec3 albedo = abs(random_vec3(seed)) * 0.9 + 0.1;
    float fuzz = random(seed) * 0.5;
    float refraction_index = mix(1.1, 2.4, random(seed));

    if (material_type == DISCO) {
        fuzz = int(random(vec4(seed, fuzz)) * 24.0) + 8.0;

        int rot_x = int(random(vec4(seed, refraction_index)) * 360.0);
        int rot_y = int(random(vec4(seed, rot_x)) * 360.0);
        int rot_z = int(random(vec4(seed, rot_y)) * 360.0);
        refraction_index = rot_x << 20 | rot_y << 10 | rot_z;
    }

    return Material(material_type, albedo, fuzz, refraction_index);
}

Sphere random_sphere(in vec3 seed) {
    vec2 seed_pos = vec2(seed.z / sqrt(MAX_SPHERES), mod(seed.z, sqrt(MAX_SPHERES))) - sqrt(MAX_SPHERES) / 2.0;
    vec3 center = vec3(seed_pos.x, 0.0, seed_pos.y) * 1.25 + random_vec3(seed) * 0.4; // + vec3(0.01, 0.0, -1.2);
    float radius = mix(0.2, 0.5, random(center + seed));
    center.y = radius - 0.5;

    Material material = random_material(center);

    //    if (random(center.xz) < 0.2) {
    //        center.y += pow(abs(sin(1.5 * time)), 0.5) * 0.5;
    //    }

    return Sphere(center, radius, material);
}

Sphere[MAX_SPHERES] setup_scene(in vec2 seed) {
    Sphere spheres[MAX_SPHERES];

    for (int i = 0; i < MAX_SPHERES; i++) {
        Sphere sphere = random_sphere(vec3(seed, i));
        spheres[i] = sphere;
    }

    // Grounddddd
    spheres[int(MAX_SPHERES - 1.0)] = Sphere(vec3(0, -1000.5, 0.0), 1000.0, Material(CHECKER_BOARD, vec3(0.5, 0.8, 0.2), 0.0, 0.0));

    return spheres;
}