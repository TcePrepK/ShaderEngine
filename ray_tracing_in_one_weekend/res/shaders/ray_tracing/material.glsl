/* <ignore> */
#include "../utils/math.glsl"
#include "../utils/ray_utils.glsl"
#include "hit_record.glsl"
/* </ignore> */

bool scatter_lambertian(in Ray ray, in HitRecord hit_record, in Material material, out vec3 attenuation, out Ray scattered) {
    vec3 scatter_direction = normalize(hit_record.normal + random_timed_vec3(ray.pixel));
    if (is_zero(scatter_direction)) {
        scatter_direction = hit_record.normal;
    }
    scattered = Ray(ray.pixel, hit_record.position + scatter_direction * EPSILON, scatter_direction);
    attenuation = material.albedo;
    return true;
}

bool scatter_metal(in Ray ray, in HitRecord hit_record, in Material material, out vec3 attenuation, out Ray scattered) {
    vec3 fuzz_vec = random_timed_vec3(ray.pixel) * material.fuzz;
    vec3 scatter_direction = normalize(reflect(ray.direction, hit_record.normal) + fuzz_vec);
    scattered = Ray(ray.pixel, hit_record.position + scatter_direction * EPSILON, scatter_direction);
    attenuation = material.albedo;
    return true;
}

bool scatter_dielectric(in Ray ray, in HitRecord hit_record, in Material material, out vec3 attenuation, out Ray scattered) {
    float cos_theta = min(dot(-ray.direction, hit_record.normal), 1.0);
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    attenuation = vec3(1.0);
    float ri = hit_record.front_face ? (1.0 / material.refraction_index) : material.refraction_index;

    vec3 new_direction = vec3(0.0);
    if (ri * sin_theta > 1.0 || reflectance(cos_theta, ri) > random(vec3(ray.pixel, time))) {
        new_direction = reflect(ray.direction, hit_record.normal);
    } else {
        new_direction = refract(ray.direction, hit_record.normal, ri);
    }

    scattered = Ray(ray.pixel, hit_record.position + new_direction * EPSILON, new_direction);
    return true;
}

bool scatter_disco(in Ray ray, in HitRecord hit_record, in Material material, out vec3 attenuation, out Ray scattered) {
    vec3 scatter_direction = normalize(hit_record.normal + random_timed_vec3(ray.pixel));
    if (is_zero(scatter_direction)) {
        scatter_direction = hit_record.normal;
    }
    scattered = Ray(ray.pixel, hit_record.position + scatter_direction * EPSILON, scatter_direction);

    vec3 n = hit_record.normal;
    n = rotate_z(n, PI / 180.0 * float(int(material.refraction_index) & 0x3FF));
    n = rotate_y(n, PI / 180.0 * float(int(material.refraction_index) >> 10 & 0x3FF));
    n = rotate_x(n, PI / 180.0 * float(int(material.refraction_index) >> 20 & 0x3FF));

    float u = atan(n.z, n.x + EPSILON) / (2.0 * PI) + 0.5; // Range [0,1]
    float v = acos(n.y) / PI;                              // Range [0,1]

    // Checkerboard tiling
    u = mod(u * material.fuzz, 1.0);
    v = mod(v * material.fuzz, 1.0);

    float iu = 0.5 - 0.5 * pow(abs(sin(2.0 * PI * u)), 0.3) * sign(0.5 - u);
    float iv = 0.5 - 0.5 * pow(abs(sin(2.0 * PI * v)), 0.3) * sign(0.5 - v);

    //    ivec3 id = ivec3(round(mod(5.0 * hit_record.normal, 1.0)));
    attenuation = material.albedo * vec3(iu * iv);
    return true;
}

bool scatter_checker_board(in Ray ray, in HitRecord hit_record, in Material material, out vec3 attenuation, out Ray scattered) {
    vec3 scatter_direction = normalize(hit_record.normal + random_timed_vec3(ray.pixel));
    if (is_zero(scatter_direction)) {
        scatter_direction = hit_record.normal;
    }
    scattered = Ray(ray.pixel, hit_record.position + scatter_direction * EPSILON, scatter_direction);

    ivec3 id = ivec3(round(mod(hit_record.position, 1.0)));
    attenuation = id.x == id.z ? vec3(1.0) : vec3(0.05);
    return true;
}