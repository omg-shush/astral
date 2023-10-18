#version 450

layout(location = 0) in vec2 fragment_uv;
layout(location = 1) in vec3 camera_position_world;
layout(location = 2) in vec3 fragment_position_world;

layout(location = 0) out vec4 out_fragment_color;

layout(set = 1, binding = 0) uniform texture3D perlin3d_texture;
layout(set = 1, binding = 1) uniform sampler perlin3d_sampler;

layout(set = 1, binding = 2) uniform SkyMaterial {
    float step_size;
    float noise_size;
    float noise_scale;
    float noise_scroll;
    float noise_bias;
    float noise_thresh;
    uint step_count;
    vec3 camera_pos;
};

// Args and return are in range 0 to 1
float density(vec3 pos) {
    pos = pos * noise_size;
    float noise = texture(sampler3D(perlin3d_texture, perlin3d_sampler), pos).r;
    // float a = texture(sampler3D(perlin3d_texture, perlin3d_sampler), pos / 2.0).r * 1.0;
    // float b = texture(sampler3D(perlin3d_texture, perlin3d_sampler), pos / 16.0).r * 4.0;
    // float c = texture(sampler3D(perlin3d_texture, perlin3d_sampler), pos / 64.0).r * 16.0;
    // float d = texture(sampler3D(perlin3d_texture, perlin3d_sampler), pos / 256.0).r * 64.0;
    // float noise = a + b + c + d;
    return max(0.0, noise - noise_thresh) * noise_scale + noise_bias;
}

vec3 world_to_box(vec3 pos) {
    vec3 box_min = vec3(-1024.0, -200.0 + 512.0, -1024.0);
    vec3 box_max = vec3(1024.0, box_min.y + 2048.0, 1024.0);
    return (pos - box_min) / (box_max - box_min);
}

void main() {
    vec3 pos = fragment_position_world + vec3(noise_scroll, 0.0, 0.0);
    vec3 dir = normalize(pos - camera_pos);

    float acc = density(world_to_box(pos)) * step_size;
    for (uint i = 0; i < step_count && acc < 2.0; i++) {
        if (acc < 0.5) {
            pos += dir * 4 * step_size;
        } else {
            pos += dir * step_size;
        }
        acc += density(world_to_box(pos)) * step_size;
    }
    float transmittance = clamp(exp(-acc), 0.0, 1.0);
    out_fragment_color = vec4(1.0, 1.0, 1.0, 1.0 - transmittance);
}
