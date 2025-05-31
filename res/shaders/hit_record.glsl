// Material type = 0 - lambertian
// Material type = 1 - metal
struct Material {
    int material_type;
    vec3 albedo;
    float metallic;
    float roughness;
};

struct HitRecord {
    vec3 position;
    vec3 normal;
    float time;
    Material material;
};

const HitRecord NO_HIT = HitRecord(vec3(0.0), vec3(0.0), 0.0, Material(0, vec3(0.0), 0.0, 0.0));