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
    int material_type = int(random(seed) * 3.0);

    vec3 albedo = abs(random_vec3(seed)) * 0.9 + 0.1;
    float fuzz = random(seed) * 0.5;
    float refraction_index = mix(1.1, 2.4, random(seed));

    return Material(material_type, albedo, fuzz, refraction_index);
}

Sphere random_sphere(in vec3 seed) {
    vec2 seed_pos = vec2(seed.z / sqrt(MAX_SPHERES), mod(seed.z, sqrt(MAX_SPHERES))) - sqrt(MAX_SPHERES) / 2.0;
    vec3 center = vec3(seed_pos.x, 0.0, seed_pos.y) * 1.25 + random_vec3(seed) * 0.4;
    float radius = mix(0.2, 0.5, random(center + seed));
    center.y = radius - 0.5;

    Material material = random_material(center);

    return Sphere(center, radius, material);
}

Sphere[MAX_SPHERES] setup_scene(in vec2 seed) {
    Sphere spheres[MAX_SPHERES];

    for (int i = 0; i < MAX_SPHERES; i++) {
        Sphere sphere = random_sphere(vec3(seed, i));
        spheres[i] = sphere;
    }

    // Grounddddd
    spheres[int(MAX_SPHERES - 1.0)] = Sphere(vec3(0, -1000.5, 0.0), 1000.0, Material(LAMBERTIAN, vec3(0.5, 0.8, 0.2), 0.0, 0.0));

    return spheres;
}