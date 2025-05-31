/* <ignore> */
#include "material.glsl"
#include "sdf_funcs.glsl"
/* </ignore> */

struct Sphere {
    vec3 center;
    float radius;
    Material material;
};

HitRecord sphere_hit(in Sphere sphere, in Ray ray) {
    float time = sdf_sphere(sphere.center, sphere.radius, ray);
    if (time > EPSILON) {
        vec3 position = ray_step(ray, time);
        vec3 normal = normalize(position - sphere.center);
        return HitRecord(position, normal, time, sphere.material);
    } else {
        return HitRecord(vec3(0.0), vec3(0.0), 0.0, sphere.material);
    }
}