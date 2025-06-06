#define LAMBERTIAN 0
#define METAL 1
#define DIELECTRIC 2
#define DISCO 3
#define CHECKER_BOARD 4

struct Material {
    int material_type;
    vec3 albedo;
    float fuzz;
    float refraction_index;
};

struct HitRecord {
    vec3 position;
    vec3 normal;
    bool front_face;
    float time;
    Material material;
};

const HitRecord NO_HIT = HitRecord(vec3(0.0), vec3(0.0), false, 0.0, Material(0, vec3(0.0), 0.0, 0.0));