/* <ignore> */
#include "utils/math.glsl"
#include "utils/ray_utils.glsl"
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
    vec3 scatter_direction = normalize(reflect(ray.direction, hit_record.normal) + random_timed_vec3(ray.pixel) * 0.05);
    scattered = Ray(ray.pixel, hit_record.position + scatter_direction * EPSILON, scatter_direction);
    attenuation = material.albedo;
    return true;
}