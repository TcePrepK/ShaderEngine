float sdf_sphere(in vec3 origin, in float radius, in vec3 ray_origin, in vec3 ray_direction) {
    vec3 oc = origin - ray_origin;
    float a = dot(ray_direction, ray_direction);
    float h = dot(ray_direction, oc);
    float c = dot(oc, oc) - radius * radius;
    float discriminant = h * h - a * c;

    if (discriminant < 0.0) {
        return -1.0;
    } else {
        float l = h / a;
        float r = sqrt(discriminant) / a;
        float root_a = l - r;
        float root_b = l + r;
        if (root_a > EPSILON) {
            return root_a;
        } else {
            return root_b;
        }
    }
}