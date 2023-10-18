#version 450

layout(location = 1) in vec3 fragment_normal;
layout(location = 2) in vec3 fragment_position_world;

layout(location = 0) out vec4 out_fragment_color;

layout(set = 1, binding = 0) uniform TerrainPlaneColoring {
    vec4 peak_color;
    vec4 flat_color;
    vec4 steep_color;
    vec4 cliff_color;
    vec4 sea_color;
    float peak_thresh;
    float cliff_thresh;
    float steep_thresh;
    float sea_thresh;
    float steep_interp;
    float cliff_interp;
};

layout(set = 1, binding = 1) uniform TerrainPlaneLighting {
    vec3 light_direction;
    vec4 diffuse_color;
    float diffuse_strength;
    vec4 ambient_color;
    float ambient_strength;
};

void main() {
    // Coloring
    vec4 color = peak_color;
    if (fragment_position_world.y < sea_thresh) {
        color = sea_color;
    } else if (fragment_position_world.y < peak_thresh) {
        if (fragment_normal.y < steep_thresh) {
            color = flat_color;
        } else if (fragment_normal.y < cliff_thresh) {
            float steepness = (fragment_normal.y - steep_thresh) / (cliff_thresh - steep_thresh);
            float interp = 1.0 - pow(steepness, steep_interp);
            color = mix(steep_color, cliff_color, interp);
        } else {
            float steepness = (fragment_normal.y - cliff_thresh) / (1.0 - cliff_thresh);
            float interp = 1.0 - pow(steepness, cliff_interp);
            color = mix(flat_color, steep_color, interp);
        }
    }

    // Lighting
    float diffuse = clamp(diffuse_strength * dot(-normalize(light_direction), fragment_normal), 0.0, 1.0);
    vec4 lighting = clamp(ambient_color * ambient_strength + diffuse_color * diffuse, 0.0, 1.0);

    out_fragment_color = color * lighting;
}
