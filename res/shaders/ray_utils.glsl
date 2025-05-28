struct Ray {
    vec3 origin;
    vec3 direction;
};

vec3 ray_step(Ray ray, float step) {
    vec3 position = ray.origin + ray.direction * step;
    return position;
}