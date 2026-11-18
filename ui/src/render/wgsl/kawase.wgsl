// ==========================================
// 1. Bindings
// ==========================================

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// ==========================================
// 2. Structs
// ==========================================

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// ==========================================
// 3. Vertex Shader
// ==========================================

@vertex
fn vs_main(model: VertexInput) -> FragmentInput {
    var out: FragmentInput;

    // From (0,0)(1,1) to (-1,-1)(1,1)
    // (center at 0,0)
    out.clip_position = vec4<f32>(model.position * 2.0 - 1.0, 0.0, 1.0);

    // Pass through to the fragment shader
    out.tex_coords = model.tex_coords;

    return out;
}

// ==========================================
// 4. Fragment Shaders
// ==========================================

// Downsample
@fragment
fn fs_down(in: FragmentInput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_diffuse));

    // Weigh the texture
    var texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords) * 4.0;

    // Sample diagonals
    let hp = texel_size * 0.5; // half pixel
    texture_color += textureSample(t_diffuse, s_diffuse, in.tex_coords + hp);
    texture_color += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(hp.x, -hp.y));
    texture_color += textureSample(t_diffuse, s_diffuse, in.tex_coords - hp);
    texture_color += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(-hp.x, hp.y));

    return texture_color / 8.0;
}

// Upsample
@fragment
fn fs_up(in: FragmentInput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_diffuse));

    // 8-tap rotated grid pattern
    var sum = vec4<f32>(0.0);
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(-texel_size.x * 2.0, 0.0));
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(texel_size.x * 2.0, 0.0));
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(0.0, -texel_size.y * 2.0));
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(0.0, texel_size.y * 2.0));

    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(-texel_size.x, -texel_size.y))
        * 2.0;
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(texel_size.x, -texel_size.y))
        * 2.0;
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(-texel_size.x, texel_size.y))
        * 2.0;
    sum += textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2(texel_size.x, texel_size.y))
        * 2.0;

    return sum / 12.0;
}
