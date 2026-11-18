// ==========================================
// 1. Bindings & Uniforms
// ==========================================

// The transform matrix calculated via glam (Affine2 converted to mat4x4 on CPU)
struct TransformUniform {
    matrix: mat4x4<f32>,
    color: vec4<f32>, // For opacity/tinting
};

@group(0) @binding(0) var<uniform> transform: TransformUniform;

// The sprite's texture and sampler
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

// (Future) Normal map will go here
// @group(1) @binding(2) var t_normal: texture_2d<f32>;


// ==========================================
// 2. Structs
// ==========================================

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>, // Optional: for opacity/tinting
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};


// ==========================================
// 3. Vertex Shader
// ==========================================

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply affine transform to the 2D local position
    // Use 0.0 for Z (2D), and 1.0 for W (translation)
    out.clip_position = transform.matrix * vec4<f32>(model.position, 0.0, 1.0);
    
    // Pass through to the fragment shader
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    
    return out;
}


// ==========================================
// 4. Fragment Shader
// ==========================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture at the given UV coordinates
    let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Multiply by the vertex color (useful for fading opacity)
    let vertex_color = texture_color * in.color;
    let transform_color = vertex_color * transform.color;
    return transform_color;
}