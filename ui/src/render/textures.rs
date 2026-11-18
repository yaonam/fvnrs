use std::path::Path;

use eframe::{egui, wgpu};
use glam::{Mat4, Quat, Vec2, Vec3};
use state::{schema::image_schema::ImageTransformSchema, screens::BaseScreenSchema};

use crate::{
    render::shaders::{TRANSFORM_UNIFORM_SIZE_PER_TEXTURE, TransformUniform},
    ui::FvnrsApp,
};

pub fn load_texture_from_path(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    path: &str,
    sampler: &wgpu::Sampler,
) -> Result<(wgpu::BindGroup, Vec2), Box<dyn std::error::Error>> {
    // Load image bytes
    let bytes = crate::assets::read_asset(path)
        .unwrap_or_else(|e| panic!("texture missing at {path:?}: {e}"));
    let img = image::load_from_memory(&bytes)
        .unwrap_or_else(|e| panic!("texture invalid at {path:?}: {e}"))
        .to_rgba8();
    let (width, height) = img.dimensions();

    let label = path;
    let view_label = format!("{label} View");
    let bind_group_label = format!("{label} Bind Group");

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        mip_level_count: 1, // Num of lower-res versions
        sample_count: 1,    // MSAA samples
        dimension: wgpu::TextureDimension::D2,
        view_formats: &[],
    });

    // Upload to GPU
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        texture.size(),
    );

    // Create BindGroup (pairs the texture with a sampler for the shader)
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&view_label),
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&bind_group_label),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok((bind_group, Vec2::new(width as f32, height as f32)))
}

fn read_target_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> (u32, u32, Vec<u8>) {
    let width = texture.width();
    let height = texture.height();
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(align) * align;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        texture.size(),
    );
    queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();

    let mapped = slice.get_mapped_range();
    let mut rgba = Vec::with_capacity((unpadded * height) as usize);
    let swap_rb = matches!(
        texture.format(),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    for row in 0..height {
        let start = (row * padded) as usize;
        let row_bytes = &mapped[start..start + unpadded as usize];
        for px in row_bytes.chunks_exact(4) {
            if swap_rb {
                rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            } else {
                rgba.extend_from_slice(px);
            }
        }
    }
    drop(mapped);
    buffer.unmap();

    (width, height, rgba)
}

fn save_thumbnail(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    resized_h: u32,
    path: &Path,
) -> image::RgbaImage {
    let full = image::RgbaImage::from_raw(width, height, rgba).unwrap();
    let resized_w = resized_h * width / height;
    let thumb = image::imageops::resize(
        &full,
        resized_w,
        resized_h,
        image::imageops::FilterType::Triangle,
    );
    thumb
        .save_with_format(path, image::ImageFormat::WebP)
        .unwrap();
    thumb
}

impl FvnrsApp {
    /// Saves a screenshot of the game target and keeps the thumbnail for the saves list.
    pub fn save_screenshot(
        &mut self,
        frame: &mut eframe::Frame,
        ctx: &egui::Context,
        path: impl AsRef<Path>,
        resized_h: u32,
    ) {
        let wgpu_state = frame
            .wgpu_render_state
            .as_ref()
            .expect("WGPU must be enabled");
        let (width, height, rgba) = read_target_rgba(
            &wgpu_state.device,
            &wgpu_state.queue,
            &self.game_target_texture,
        );
        let thumb = save_thumbnail(rgba, width, height, resized_h, path.as_ref());
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [thumb.width() as usize, thumb.height() as usize],
            thumb.as_raw(),
        );
        let path = path.as_ref().to_string_lossy().into_owned();
        let handle = ctx.load_texture(&path, color, egui::TextureOptions::LINEAR);
        self.screenshot_tex_ids.insert(path, handle);
    }

    pub fn render_textures(&mut self, frame: &mut eframe::Frame) {
        log::info!("Rendering textures");
        let wgpu_state = frame
            .wgpu_render_state
            .as_ref()
            .expect("WGPU must be enabled");
        let core_read = self.engine_core.read();

        // Get texture handles and transforms from game state
        let mut textures_and_transforms = Vec::new();
        {
            match core_read.screens.splash_screen {
                Some(_) => {
                    if let Some(splash_texture) =
                        core_read.texture_cache.get(&self.loading_splash_path)
                    {
                        textures_and_transforms.push((splash_texture.clone(), None));
                    } else {
                        log::warn!("Splash texture not found: {}", self.loading_splash_path);
                    }
                }
                None => {
                    let images = match &core_read.screens.base_screen {
                        BaseScreenSchema::Game(game_state) => &game_state.playable.images,
                        BaseScreenSchema::MainMenu(main_menu) => &main_menu.images,
                    };
                    log::info!(
                        "Rendering {} image(s)",
                        images.values().map(|v| v.len()).sum::<usize>()
                    );
                    for (_id, images) in images.iter() {
                        for image in images.iter() {
                            if let Some(texture) = core_read.texture_cache.get(image.path.as_str())
                            {
                                log::info!(
                                    "  {} opacity={:?}",
                                    image.path,
                                    image.transform.opacity
                                );
                                textures_and_transforms
                                    .push((texture.clone(), Some(&image.transform)));
                            } else {
                                log::warn!("Texture not found: {}", image.path);
                            }
                        }
                    }
                }
            }
        }

        // Sort by z_index
        textures_and_transforms
            .sort_by_key(|(_, transform)| transform.and_then(|t| t.z_index).unwrap_or(0));

        // Draw textures and apply transforms
        let mut encoder =
            wgpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Game Textures Encoder"),
                });

        // Clear pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game Textures Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.game_target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default() // depth_stencil_attachment, timestamp_writes, occlusion_query_set, multiview_mask: None,
        });

        // Now draw each texture
        for (i, (bind_group_size, transform)) in textures_and_transforms.iter().enumerate() {
            // Slot in buffer
            let dynamic_offset = TRANSFORM_UNIFORM_SIZE_PER_TEXTURE * i as u64;

            // Calculate transform matrix
            let transform_uniform = Self::calc_transform_uniform(
                bind_group_size.1,
                transform.unwrap_or(&ImageTransformSchema::default()),
            );
            wgpu_state.queue.write_buffer(
                &self.transform_bind_group.buffer,
                dynamic_offset,
                bytemuck::cast_slice(&[transform_uniform]),
            );

            // Set pipeline
            render_pass.set_pipeline(&self.pipelines_and_layouts.pipeline);

            // Set bind group(s)
            render_pass.set_bind_group(
                0,
                &self.transform_bind_group.bind_group,
                &[dynamic_offset as u32],
            );
            // Texture + sampler
            render_pass.set_bind_group(1, &bind_group_size.0, &[]);
            // Vertex buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Draw
            render_pass.draw(0..6, 0..1); // 6 vertices, 1 instance
        }

        // Submit commands
        drop(render_pass);
        wgpu_state.queue.submit(Some(encoder.finish()));

        // Render Kawase blur here too for now
        drop(core_read);
        self.render_kawase_blur(&mut *frame);
    }

    fn render_kawase_blur(&mut self, frame: &mut eframe::Frame) {
        let wgpu_state = frame
            .wgpu_render_state
            .as_ref()
            .expect("WGPU must be enabled");

        // Draw textures and apply transforms
        let mut encoder =
            wgpu_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Kawase Blur Encoder"),
                });

        // ==========================================
        // Downsample Passes
        // ==========================================

        for (i, kawase_target) in self.post_process_resources.targets.iter().enumerate() {
            // Render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Kawase Downsample Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &kawase_target.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default() // depth_stencil_attachment, timestamp_writes, occlusion_query_set, multiview_mask: None,
            });

            // Set pipeline
            render_pass.set_pipeline(&self.post_process_resources.down_pipeline);

            // Texture + sampler (use game target or prev)
            let texture_bind_group = if i == 0 {
                &self.game_target_bind_group
            } else {
                &self.post_process_resources.targets[i - 1].bind_group
            };
            render_pass.set_bind_group(0, texture_bind_group, &[]);
            // Vertex buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Draw
            render_pass.draw(0..6, 0..1); // 6 vertices, 1 instance

            // Submit commands
            drop(render_pass);
        }

        // ==========================================
        // Upsample Passes
        // ==========================================

        let kawase_targets_len = self.post_process_resources.targets.len();
        for i in (0..kawase_targets_len).rev() {
            let source = &self.post_process_resources.targets[i];
            let dest_view = if i == 0 {
                &self.post_process_resources.blurred_target.view
            } else {
                &self.post_process_resources.targets[i - 1].view
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Kawase Upsample Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dest_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            // Set pipeline
            render_pass.set_pipeline(&self.post_process_resources.up_pipeline);

            // Texture
            render_pass.set_bind_group(0, &source.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Draw
            render_pass.draw(0..6, 0..1); // 6 vertices, 1 instance

            // Submit commands
            drop(render_pass);
        }

        wgpu_state.queue.submit(Some(encoder.finish()));
    }

    fn calc_transform_uniform(size: Vec2, transform: &ImageTransformSchema) -> TransformUniform {
        use crate::ui::{TARGET_HEIGHT, TARGET_WIDTH};

        // Maps game space (0..1920, 0..1080, Y-up) → clip space (-1..1)
        let proj = Mat4::orthographic_rh(0.0, TARGET_WIDTH, 0.0, TARGET_HEIGHT, -1.0, 1.0);

        // Resolve defaults
        let scale = transform.scale.unwrap_or(1.0);
        let pos = match transform.position {
            None => size * 0.5,
            Some(pos) => Vec2::from_array(pos),
        };
        let rotation = transform.rotation.unwrap_or(0.0);
        let pivot = transform.pivot.unwrap_or([0.5, 0.5]);

        // 1. Shift pivot to origin (-pivot_offset)
        // 2. Scale (uniform scale * sprite pixel size, so the unit quad covers the sprite)
        // 3. Rotate (Around Z axis for 2D)
        // 4. Translate to final position in game space
        let model = Mat4::from_translation(Vec3::new(pos.x, pos.y, 0.0))
            * Mat4::from_quat(Quat::from_rotation_z(rotation))
            * Mat4::from_scale(Vec3::new(scale * size.x, scale * size.y, 1.0))
            * Mat4::from_translation(Vec3::new(-pivot[0], -pivot[1], 0.0));

        let matrix = (proj * model).to_cols_array();

        let opacity = match transform.opacity {
            Some(o) => o,
            None => 1.0, // WGSL expects 0.0 - 1.0
        };

        TransformUniform {
            matrix,
            color: [1.0, 1.0, 1.0, opacity],
        }
    }
}
