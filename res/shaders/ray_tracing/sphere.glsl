/* <ignore> */
#include "material.glsl"
#include "utils/sdf_funcs.glsl"
/* </ignore> */

struct Sphere {
    vec3 center;
    float radius;
    Material material;
};

HitRecord sphere_hit(in Sphere sphere, in Ray ray) {
    float time = sdf_sphere(sphere.center, sphere.radius, ray.origin, ray.direction);
    if (time > EPSILON) {
        vec3 position = ray_step(ray, time);
        vec3 normal = normalize(position - sphere.center);
        return HitRecord(position, normal, time, sphere.material);
    } else {
        return NO_HIT;
    }
}