// Random hash generation code is completely from https://stackoverflow.com/a/17479300
// With a tiny bit of modification in hash functions

// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
uint hash(in uint x) {
    x += (x << 10u);
    x ^= (x >> 6u);
    x += (x << 3u);
    x ^= (x >> 11u);
    x += (x << 15u);
    return x;
}

// Compound versions of the hashing algorithm I whipped together.
uint hash(in uvec2 v) { return hash(v.x ^ hash(v.y)); }
uint hash(in uvec3 v) { return hash(v.x ^ hash(v.y ^ hash(v.z))); }
uint hash(in uvec4 v) { return hash(v.x ^ hash(v.y ^ hash(v.z ^ hash(v.w)))); }

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
float float_construct(in uint m) {
    const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
    const uint ieeeOne = 0x3F800000u;      // 1.0 in IEEE binary32

    m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    m |= ieeeOne;                          // Add fractional part to 1.0

    float f = uintBitsToFloat(m);          // Range [1:2]
    return f - 1.0;                        // Range [0:1]
}

// Pseudo-random value in half-open range [0:1].
float random(in float x) { return float_construct(hash(floatBitsToUint(x))); }
float random(in vec2 v) { return float_construct(hash(floatBitsToUint(v))); }
float random(in vec3 v) { return float_construct(hash(floatBitsToUint(v))); }
float random(in vec4 v) { return float_construct(hash(floatBitsToUint(v))); }

// End of stolen code

vec2 random_vec2(in vec2 seed) {
    float x = random(seed) * 2.0 - 1.0;
    float y = random(vec3(seed, x)) * 2.0 - 1.0;

    return vec2(x, y);
}

vec2 random_timed_vec2(in vec2 seed) {
    float x = random(vec3(seed, time)) * 2.0 - 1.0;
    float y = random(vec3(seed, x)) * 2.0 - 1.0;

    return vec2(x, y);
}

vec3 random_vec3(in vec2 seed) {
    float x = random(seed) * 2.0 - 1.0;
    float y = random(vec3(seed, x)) * 2.0 - 1.0;
    float z = random(vec3(seed, y)) * 2.0 - 1.0;

    return vec3(x, y, z);
}

vec3 random_timed_vec3(in vec2 seed) {
    float x = random(vec3(seed, time)) * 2.0 - 1.0;
    float y = random(vec3(seed, x)) * 2.0 - 1.0;
    float z = random(vec3(seed, y)) * 2.0 - 1.0;

    return vec3(x, y, z);
}

bool is_zero(in float x) { return abs(x) < EPSILON; }
bool is_zero(in vec2 v) { return is_zero(v.x) && is_zero(v.y); }
bool is_zero(in vec3 v) { return is_zero(v.x) && is_zero(v.y) && is_zero(v.z); }
bool is_zero(in vec4 v) { return is_zero(v.x) && is_zero(v.y) && is_zero(v.z) && is_zero(v.w); }