/* <ignore> */
#include "ray_utils.glsl"
/* </ignore> */

float sdf_sphere(in vec3 origin, in float radius, in Ray ray) {
    vec3 oc = origin - ray.origin;
    float a = dot(ray.direction, ray.direction);
    float h = dot(ray.direction, oc);
    float c = dot(oc, oc) - radius * radius;
    float discriminant = h * h - a * c;

    if (discriminant < 0.0) {
        return -1.0;
    } else {
        float root_a = (h - sqrt(discriminant)) / a;
        float root_b = (h + sqrt(discriminant)) / a;
        if (root_a > EPSILON) {
            return root_a;
        } else {
            return root_b;
        }
    }
}