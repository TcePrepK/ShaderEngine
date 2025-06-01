struct Ray {
    vec2 pixel;
    vec3 origin;
    vec3 direction;
};

vec3 ray_step(in Ray ray, in float step) {
    vec3 position = ray.origin + ray.direction * step;
    return position;
}

vec3 sky_color(in Ray ray) {
    vec3 unit_direction = normalize(ray.direction);
    float t = 0.5 * (unit_direction.y + 1.0);
    return mix(vec3(1.0), vec3(0.5, 0.7, 1.0), t);
}

const float verticel_fov = 90.0;
const vec3 look_at = vec3(0.0, 0.0, 0.0);
const vec3 up = vec3(0.0, 1.0, 0.0);

Ray camera_ray(in vec2 pixel, in vec2 resolution, in float time) {
    float rotation_time = time * 0.5;
    vec3 look_from = vec3(cos(rotation_time), 0.5, sin(rotation_time)) * 3.0; // Camera position

    float focal_length = length(look_from - look_at);
    float theta = verticel_fov * 3.14159265 / 180.0;
    float h = tan(theta / 2.0);

    float viewport_height = 2.0 * h * focal_length;
    float viewport_width = viewport_height * resolution.x / resolution.y;

    vec3 w = normalize(look_from - look_at);
    vec3 u = normalize(cross(up, w));
    vec3 v = cross(w, u);

    vec3 viewport_u = viewport_width * u;
    vec3 viewport_v = viewport_height * -v;

    vec3 viewport_upper_left_corner = look_from - focal_length * w - (viewport_u + viewport_v) * 0.5;
    vec3 pixel_location = viewport_upper_left_corner + pixel.x * viewport_u + pixel.y * viewport_v;

    return Ray(pixel, look_from, normalize(pixel_location - look_from));
}