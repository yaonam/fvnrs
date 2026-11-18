use std::num::NonZeroU64;

use eframe::wgpu;
use wgpu::util::DeviceExt;

const MAX_TEXTURES: u64 = 128;
pub(crate) const TRANSFORM_UNIFORM_SIZE_PER_TEXTURE: u64 = 256;
const TRANSFORM_UNIFORM_SIZE: u64 = MAX_TEXTURES * TRANSFORM_UNIFORM_SIZE_PER_TEXTURE;

/// Matches the TransformUniform in `basic.wgsl`
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TransformUniform {
    pub(crate) matrix: [f32; 16],
    pub(crate) color: [f32; 4],
}

/// Matches the vertex layout in `basic.wgsl` and `VertexBufferLayout` below (32 bytes).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

pub(crate) struct BasicPipelineAndLayouts {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) transform_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
}

pub(crate) struct BufferBindGroup {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

/// One ping-pong / pyramid level for kawase blur.
pub(crate) struct RenderTarget {
    pub(crate) view: wgpu::TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
}

pub(crate) struct KawaseResources {
    pub(crate) down_pipeline: wgpu::RenderPipeline,
    pub(crate) up_pipeline: wgpu::RenderPipeline,
    /// Downsample pyramid: `[0]` = half res, `[1]` = quarter, ...
    pub(crate) targets: Vec<RenderTarget>,
    /// Full-res result after the final upsample
    pub(crate) blurred_target: RenderTarget,
}

pub(crate) fn create_basic_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Basic Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("wgsl/basic.wgsl").into()),
    })
}

pub(crate) fn create_kawase_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Kawase Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("wgsl/kawase.wgsl").into()),
    })
}

pub(crate) fn create_pipelines_and_layouts(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> BasicPipelineAndLayouts {
    // Create Shaders
    let basic_shader = create_basic_shader(device);

    // Create Layouts
    // These define the inputs passed to the shaders (uniforms and textures).
    // We need to persist these in the App state for loading textures.
    let transform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Transform Layout (Group 0)"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(
                        NonZeroU64::new(TRANSFORM_UNIFORM_SIZE_PER_TEXTURE).unwrap(),
                    ),
                },
                count: None,
            }],
        });
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Layout (Group 1)"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // t_diffuse
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, // s_diffuse
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    // Build Pipeline
    // This is just needed for setting up the pipeline and can be dropped after.
    let basic_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Basic Pipeline Layout"),
        bind_group_layouts: &[
            Some(&transform_bind_group_layout),
            Some(&texture_bind_group_layout),
        ],
        ..Default::default()
    });

    let basic_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Basic Pipeline"),
        layout: Some(&basic_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &basic_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                // Vertex inputs
                array_stride: 32, // 8 + 8 + 16 = 32 bytes
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        // Position
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        // Texture coordinates
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 8,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        // Color
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 2,
                    },
                ],
            }],
        },
        primitive: Default::default(),
        depth_stencil: None, // 2D target, no depth
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &basic_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format, // MUST match what the UI uses!
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None, // multiview / VR
        cache: None,
    });

    BasicPipelineAndLayouts {
        pipeline: basic_pipeline,
        transform_bind_group_layout,
        texture_bind_group_layout,
    }
}

pub(crate) fn create_kawase_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    kawase_shader: &wgpu::ShaderModule,
    kawase_pipeline_layout: &wgpu::PipelineLayout,
    entry_point: &str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Kawase Pipeline"),
        layout: Some(&kawase_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &kawase_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                // Vertex inputs
                array_stride: 32, // 8 + 8 + 16 = 32 bytes
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        // Position
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        // Texture coordinates
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 8,
                        shader_location: 1,
                    },
                ],
            }],
        },
        primitive: Default::default(),
        depth_stencil: None, // 2D target, no depth
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &kawase_shader,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format, // MUST match what the UI uses!
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None, // multiview / VR
        cache: None,
    })
}

pub(crate) fn create_transform_buffer_and_bind_group(
    device: &wgpu::Device,
    transform_bind_group_layout: &wgpu::BindGroupLayout,
) -> BufferBindGroup {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Transform Buffer"),
        size: TRANSFORM_UNIFORM_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Transform Bind Group"),
        layout: transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: Some(NonZeroU64::new(TRANSFORM_UNIFORM_SIZE_PER_TEXTURE).unwrap()),
            }),
        }],
    });

    BufferBindGroup { buffer, bind_group }
}

pub(crate) fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Default Texture Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

/// Creates a vertex buffer using a quad of vertices (two triangles).
/// (0, 0) ---------- (1, 0)
///   |              |
///   |              |
///   |              |
///   |              |
/// (0, 1) ---------- (1, 1)
pub(crate) fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let white = [1.0, 1.0, 1.0, 1.0];
    let vertices = [
        Vertex {
            position: [0.0, 0.0],
            tex_coords: [0.0, 1.0],
            color: white,
        },
        Vertex {
            position: [1.0, 0.0],
            tex_coords: [1.0, 1.0],
            color: white,
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 0.0],
            color: white,
        },
        Vertex {
            position: [0.0, 0.0],
            tex_coords: [0.0, 1.0],
            color: white,
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 0.0],
            color: white,
        },
        Vertex {
            position: [0.0, 1.0],
            tex_coords: [0.0, 0.0],
            color: white,
        },
    ];

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_kawase_target(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    label: &str,
) -> RenderTarget {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&format!("{label} View")),
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{label} Bind Group")),
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    RenderTarget { view, bind_group }
}

/// Builds half/quarter/... pyramid targets plus a full-res blurred output.
pub(crate) fn create_post_process_resources(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    full_width: u32,
    full_height: u32,
    kawase_passes: usize,
) -> KawaseResources {
    let mut targets = Vec::with_capacity(kawase_passes);
    let mut width = full_width;
    let mut height = full_height;

    for i in 0..kawase_passes {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        targets.push(create_kawase_target(
            device,
            format,
            texture_bind_group_layout,
            sampler,
            width,
            height,
            &format!("Kawase Level {i} ({width}x{height})"),
        ));
    }

    let blurred_target = create_kawase_target(
        device,
        format,
        texture_bind_group_layout,
        sampler,
        full_width,
        full_height,
        "Kawase Blurred Target",
    );

    let kawase_shader = create_kawase_shader(device);
    let kawase_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Kawase Pipeline Layout"),
        bind_group_layouts: &[Some(&texture_bind_group_layout)],
        ..Default::default()
    });

    let down_pipeline = create_kawase_pipeline(
        device,
        format,
        &kawase_shader,
        &kawase_pipeline_layout,
        "fs_down",
    );
    let up_pipeline = create_kawase_pipeline(
        device,
        format,
        &kawase_shader,
        &kawase_pipeline_layout,
        "fs_up",
    );

    KawaseResources {
        down_pipeline,
        up_pipeline,
        targets,
        blurred_target,
    }
}
