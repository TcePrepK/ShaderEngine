/* <ignore> */
#include "material.glsl"
#include "../utils/sdf_funcs.glsl"
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
        bool front_face = dot(ray.direction, normal) < 0.0;
        normal = front_face ? normal : -normal;
        return HitRecord(position, normal, front_face, time, sphere.material);
    } else {
        return NO_HIT;
    }
}