use crate::features::model3d::math::{Mat4, look_at, multiply, perspective};
use crate::features::model3d::types::{BoundingBox, Model3dState, ShadingMode};
use iced::advanced::graphics::Viewport;
use iced::mouse;
use iced::widget::shader::{self, Action};
use iced::{Element, Length, Rectangle};
use iced_wgpu::primitive::Pipeline;
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use std::sync::Arc;

pub struct Model3dCallbacks<'a, Message> {
    pub on_drag_start: Box<dyn Fn(iced::Point) -> Message + 'a>,
    pub on_drag_move: Box<dyn Fn(iced::Point) -> Message + 'a>,
    pub on_drag_end: Message,
    pub on_zoom: Box<dyn Fn(f32) -> Message + 'a>,
}

pub struct Model3dWidget<'a, Message> {
    state: &'a Model3dState,
    vertices: Arc<[f32]>,
    indices: Arc<[u32]>,
    bounds: BoundingBox,
    callbacks: Model3dCallbacks<'a, Message>,
}

impl<'a, Message: Clone + 'static> Model3dWidget<'a, Message> {
    pub fn new(
        state: &'a Model3dState,
        vertices: Arc<[f32]>,
        indices: Arc<[u32]>,
        bounds: BoundingBox,
        callbacks: Model3dCallbacks<'a, Message>,
    ) -> Self {
        Self {
            state,
            vertices,
            indices,
            bounds,
            callbacks,
        }
    }
}

pub struct Model3dProgram<'a, Message> {
    widget: Model3dWidget<'a, Message>,
}

impl<'a, Message: Clone + 'static> shader::Program<Message> for Model3dProgram<'a, Message> {
    type State = ();
    type Primitive = Model3dPrimitive;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if let iced::Event::Mouse(mouse_event) = event {
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let absolute_point = iced::Point::new(bounds.x + pos.x, bounds.y + pos.y);
                        return Some(Action::publish((self.widget.callbacks.on_drag_start)(
                            absolute_point,
                        )));
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if self.widget.state.is_dragging
                        && let Some(pos) = cursor.position()
                    {
                        return Some(Action::publish((self.widget.callbacks.on_drag_move)(pos)));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if self.widget.state.is_dragging {
                        return Some(Action::publish(self.widget.callbacks.on_drag_end.clone()));
                    }
                }
                mouse::Event::WheelScrolled { delta } if cursor.is_over(bounds) => {
                    let factor = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            if *y > 0.0 {
                                0.9
                            } else {
                                1.1
                            }
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            if *y > 0.0 {
                                0.95
                            } else {
                                1.05
                            }
                        }
                    };
                    return Some(Action::publish((self.widget.callbacks.on_zoom)(factor)));
                }
                _ => {}
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        // Calculate MVP matrix
        let yaw = self.widget.state.yaw;
        let pitch = self.widget.state.pitch;
        let dist = self.widget.state.distance;

        let eye_x = dist * pitch.cos() * yaw.sin();
        let eye_y = dist * pitch.sin();
        let eye_z = dist * pitch.cos() * yaw.cos();

        let view = look_at([eye_x, eye_y, eye_z], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        Model3dPrimitive {
            vertices: Arc::clone(&self.widget.vertices),
            indices: Arc::clone(&self.widget.indices),
            view_matrix: view,
            shading: self.widget.state.shading,
            show_grid: self.widget.state.show_grid,
            bounds: self.widget.bounds,
            camera_pos: [eye_x, eye_y, eye_z],
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.widget.state.is_dragging {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::Grab
        }
    }
}

#[derive(Debug)]
pub struct Model3dPrimitive {
    vertices: Arc<[f32]>,
    indices: Arc<[u32]>,
    view_matrix: Mat4,
    shading: ShadingMode,
    show_grid: bool,
    bounds: BoundingBox,
    camera_pos: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Uniforms {
    mvp: [f32; 16],
    camera_pos: [f32; 4],
    shading_params: [f32; 4],
    _pad: [f32; 8],
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

pub struct Model3dPipeline {
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: Option<wgpu::RenderPipeline>,
    grid_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth_texture: Option<(wgpu::Texture, wgpu::TextureView, u32, u32)>,
    vertex_buffer: Option<(wgpu::Buffer, usize)>,
    index_buffer: Option<(wgpu::Buffer, usize)>,
    grid_vertex_buffer: Option<wgpu::Buffer>,
    grid_vertex_count: u32,
}

const WGSL_SHADER: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
    camera_pos: vec4<f32>,
    shading_params: vec4<f32>,
    _pad: mat2x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.world_pos = model.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let shading_mode = u32(uniforms.shading_params.x);
    if (shading_mode == 2u) {
        // Wireframe: Sleek technical cyan
        return vec4<f32>(0.25, 0.85, 1.0, 1.0);
    }

    var normal = normalize(in.normal);
    if (shading_mode == 0u) {
        // Flat shading via derivative
        let dx = dpdx(in.world_pos);
        let dy = dpdy(in.world_pos);
        normal = normalize(cross(dx, dy));
    }

    let light_dir1 = normalize(vec3<f32>(0.5, 1.0, 0.8));
    let light_dir2 = normalize(vec3<f32>(-0.6, -0.3, -0.5));
    let light_dir3 = normalize(vec3<f32>(0.0, -1.0, 0.0)); // subtle bounce from floor

    let diff1 = max(dot(normal, light_dir1), 0.0);
    let diff2 = max(dot(normal, light_dir2), 0.0) * 0.45;
    let diff3 = max(dot(normal, light_dir3), 0.0) * 0.20;

    // View direction and halfway vector for specular highlight
    let V = normalize(uniforms.camera_pos.xyz - in.world_pos);
    let H1 = normalize(light_dir1 + V);
    let spec1 = pow(max(dot(normal, H1), 0.0), 32.0) * 0.22;

    // Fresnel rim light for silhouette edge definition on bright backgrounds
    let rim = pow(1.0 - max(dot(normal, V), 0.0), 3.0) * 0.15;

    // Hemisphere ambient: warm skylight above, cool subtle bounce below
    let hemi_ambient = mix(vec3<f32>(0.35, 0.40, 0.48), vec3<f32>(0.65, 0.70, 0.78), normal.y * 0.5 + 0.5) * 0.55;

    let base_color = vec3<f32>(0.4, 0.65, 0.95); // Sleek cyan/blue
    let diffuse_lighting = diff1 * 0.65 + diff2 + diff3;
    let final_color = base_color * (hemi_ambient + diffuse_lighting) + vec3<f32>(1.0, 1.0, 1.0) * spec1 + vec3<f32>(0.8, 0.9, 1.0) * rim;

    return vec4<f32>(final_color, 1.0);
}

@vertex
fn vs_grid(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.world_pos = model.position;
    return out;
}

@fragment
fn fs_grid(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Distance-based horizon fade
    let dist_xz = length(vec2<f32>(in.world_pos.x, in.world_pos.z));
    let max_radius = in.normal.y; // stored grid extent radius
    let fade = clamp(1.0 - (dist_xz / max_radius), 0.0, 1.0);
    let smooth_fade = fade * fade; // smooth falloff

    // 2. Analytic Ground Contact Shadow under model
    let contact_radius = max_radius * 0.40;
    let contact_norm = clamp(dist_xz / contact_radius, 0.0, 1.0);
    let contact_shadow = smoothstep(0.0, 1.0, contact_norm);
    let shadow_intensity = mix(0.35, 0.0, 1.0 - contact_shadow); // darker underneath

    let base_alpha = in.normal.x;
    let alpha = (base_alpha + shadow_intensity) * smooth_fade;
    return vec4<f32>(0.35, 0.45, 0.58, alpha);
}
"#;

impl Pipeline for Model3dPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model3d_shader"),
            source: wgpu::ShaderSource::Wgsl(WGSL_SHADER.into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model3d_uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("model3d_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("model3d_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model3d_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: (6 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: (3 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        };

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model3d_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: std::slice::from_ref(&vertex_layout),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: depth_stencil.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let wireframe_pipeline = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            Some(
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("model3d_wireframe_pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: std::slice::from_ref(&vertex_layout),
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Line,
                        conservative: false,
                    },
                    depth_stencil: depth_stencil.clone(),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                }),
            )
        } else {
            None
        };

        // Grid pipeline uses line list
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model3d_grid_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_grid"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_grid"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            wireframe_pipeline,
            grid_pipeline,
            uniform_buffer,
            bind_group,
            depth_texture: None,
            vertex_buffer: None,
            index_buffer: None,
            grid_vertex_buffer: None,
            grid_vertex_count: 0,
        }
    }
}

impl iced_wgpu::primitive::Primitive for Model3dPrimitive {
    type Pipeline = Model3dPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let width = viewport.physical_width().max(1);
        let height = viewport.physical_height().max(1);

        let aspect = bounds.width.max(1.0) / bounds.height.max(1.0);
        let proj = perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, 1000.0);
        let mvp = multiply(&proj, &self.view_matrix);

        let shading_mode = match self.shading {
            ShadingMode::Flat => 0u32,
            ShadingMode::Smooth => 1u32,
            ShadingMode::Wireframe => 2u32,
        };

        let uniforms = Uniforms {
            mvp,
            camera_pos: [
                self.camera_pos[0],
                self.camera_pos[1],
                self.camera_pos[2],
                1.0,
            ],
            shading_params: [shading_mode as f32, 0.0, 0.0, 0.0],
            _pad: [0.0; 8],
        };

        queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Ensure depth texture exists and matches swapchain dimensions
        let recreate_depth = match &pipeline.depth_texture {
            Some((_, _, w, h)) => *w != width || *h != height,
            None => true,
        };

        if recreate_depth {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("model3d_depth_texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            pipeline.depth_texture = Some((texture, view, width, height));
        }

        // Upload vertex buffer if changed
        let vert_bytes = bytemuck::cast_slice(&self.vertices);
        let need_vert_reupload = match &pipeline.vertex_buffer {
            Some((_, len)) => *len != vert_bytes.len(),
            None => true,
        };
        if need_vert_reupload {
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model3d_vertex_buffer"),
                contents: vert_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });
            pipeline.vertex_buffer = Some((buf, vert_bytes.len()));
        }

        // Upload index buffer if changed
        let idx_bytes = bytemuck::cast_slice(&self.indices);
        let need_idx_reupload = match &pipeline.index_buffer {
            Some((_, len)) => *len != idx_bytes.len(),
            None => true,
        };
        if need_idx_reupload {
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model3d_index_buffer"),
                contents: idx_bytes,
                usage: wgpu::BufferUsages::INDEX,
            });
            pipeline.index_buffer = Some((buf, idx_bytes.len()));
        }

        // Generate and upload ground grid buffer
        if self.show_grid && pipeline.grid_vertex_buffer.is_none() {
            let max_dim = self.bounds.max_dimension().max(1.0);
            let half_y = -self.bounds.size()[1] * 0.5; // floor level
            let grid_extent = max_dim * 1.5;
            let steps = 14;
            let step_size = (grid_extent * 2.0) / (steps as f32);

            let mut grid_verts = Vec::new();
            for i in 0..=steps {
                let coord = -grid_extent + (i as f32) * step_size;
                let is_center = i == steps / 2;
                let intensity = if is_center { 0.45 } else { 0.18 };

                // Line along Z: (coord, half_y, -grid_extent) -> (coord, half_y, grid_extent)
                grid_verts.extend_from_slice(&[
                    coord,
                    half_y,
                    -grid_extent,
                    intensity,
                    grid_extent,
                    0.0,
                ]);
                grid_verts.extend_from_slice(&[
                    coord,
                    half_y,
                    grid_extent,
                    intensity,
                    grid_extent,
                    0.0,
                ]);

                // Line along X: (-grid_extent, half_y, coord) -> (grid_extent, half_y, coord)
                grid_verts.extend_from_slice(&[
                    -grid_extent,
                    half_y,
                    coord,
                    intensity,
                    grid_extent,
                    0.0,
                ]);
                grid_verts.extend_from_slice(&[
                    grid_extent,
                    half_y,
                    coord,
                    intensity,
                    grid_extent,
                    0.0,
                ]);
            }

            let grid_bytes = bytemuck::cast_slice(&grid_verts);
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model3d_grid_buffer"),
                contents: grid_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });
            pipeline.grid_vertex_count = (grid_verts.len() / 6) as u32;
            pipeline.grid_vertex_buffer = Some(buf);
        }
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let depth_view = match &pipeline.depth_texture {
            Some((_, view, _, _)) => view,
            None => return,
        };
        let (vertex_buf, _) = match &pipeline.vertex_buffer {
            Some(v) => v,
            None => return,
        };
        let (index_buf, _) = match &pipeline.index_buffer {
            Some(i) => i,
            None => return,
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("model3d_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );

        // 1. Draw Model Mesh
        let active_pipeline = if self.shading == ShadingMode::Wireframe {
            pipeline
                .wireframe_pipeline
                .as_ref()
                .unwrap_or(&pipeline.render_pipeline)
        } else {
            &pipeline.render_pipeline
        };

        pass.set_pipeline(active_pipeline);
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buf.slice(..));
        pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        // 2. Draw Ground Grid if enabled
        if self.show_grid
            && let Some(grid_buf) = &pipeline.grid_vertex_buffer
        {
            pass.set_pipeline(&pipeline.grid_pipeline);
            pass.set_bind_group(0, &pipeline.bind_group, &[]);
            pass.set_vertex_buffer(0, grid_buf.slice(..));
            pass.draw(0..pipeline.grid_vertex_count, 0..1);
        }
    }
}

pub fn model3d_widget<'a, Message: Clone + 'static>(
    state: &'a Model3dState,
    vertices: Arc<[f32]>,
    indices: Arc<[u32]>,
    bounds: BoundingBox,
    callbacks: Model3dCallbacks<'a, Message>,
) -> Element<'a, Message> {
    let widget = Model3dWidget::new(state, vertices, indices, bounds, callbacks);

    shader::Shader::new(Model3dProgram { widget })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
