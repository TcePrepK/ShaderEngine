#version 460 core

uniform vec2 resolution;
uniform float time;

out vec4 out_color;

#include "ray_utils.glsl"

float smootherstep(float edge0, float edge1, float x) {
    x = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
}

vec2 smootherstep(vec2 x) {
    return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
}

vec2 hash(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
}

vec2 perlin(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = smootherstep(f);

    vec2 hash_down = mix(hash(i + vec2(0.0, 0.0)), hash(i + vec2(1.0, 0.0)), u.x);
    vec2 hash_up = mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x);
    return mix(hash_down, hash_up, u.y);
}

Ray camera_ray(vec2 pixel) {
    float viewport_height = 2.0;
    float viewport_width = viewport_height * resolution.x / resolution.y;

    float focal_length = 1.0;
    vec3 camera_center = vec3(0.0, 0.0, 0.0);

    vec3 viewport_u = vec3(viewport_width, 0.0, 0.0);
    vec3 viewport_v = vec3(0.0, -viewport_height, 0.0);

    vec3 viewport_upper_left_corner = camera_center - vec3(0.0, 0.0, focal_length) - (viewport_u + viewport_v) / 2.0;
    vec3 pixel_location = viewport_upper_left_corner + pixel.x * viewport_u + pixel.y * viewport_v;

    return Ray(camera_center, normalize(pixel_location));
}

vec3 sky_color(Ray ray) {
    vec3 unit_direction = normalize(ray.direction);
    float t = 0.5 * (unit_direction.y + 1.0);
    return mix(vec3(1.0), vec3(0.5, 0.7, 1.0), t);
}

bool hit_sphere(in vec3 origin, in float radius, in Ray ray) {
    vec3 oc = origin - ray.origin;
    float a = dot(ray.direction, ray.direction);
    float b = -2.0 * dot(oc, ray.direction);
    float c = dot(oc, oc) - radius * radius;
    float discriminant = b * b - 4.0 * a * c;
    if (discriminant < 0.0) {
        return false;
    }
    float t = (-b - sqrt(discriminant)) / (2.0 * a);
    if (t > 0.0) {
        return true;
    }
    t = (-b + sqrt(discriminant)) / (2.0 * a);
    if (t > 0.0) {
        return true;
    }
    return false;
}

void main() {
    vec2 pixel = gl_FragCoord.xy / resolution;
    pixel.y = 1.0 - pixel.y;

    Ray ray = camera_ray(pixel);
    vec3 color = sky_color(ray);

    if (hit_sphere(vec3(0.0, 0.0, -1.0), 0.5, ray)) {
        color = vec3(1.0, 0.0, 0.0);
    }

    out_color = vec4(color, 1.0);
}