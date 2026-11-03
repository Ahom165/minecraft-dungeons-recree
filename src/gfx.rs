//! Renderer wgpu : monde (cuboïdes texturés, éclairage ponctuel + brouillard),
//! billboards de particules, interface ortho, capture d'écran.

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::vertex::{DynVertex, FxInstance, UiVertex};

const WORLD_WGSL: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    fog: vec4<f32>,
    sun: vec4<f32>,
    sun_color: vec4<f32>,
    info: vec4<f32>,
    lights: array<vec4<f32>, 16>,
    light_colors: array<vec4<f32>, 16>,
};
struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) nrm: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tint: vec4<f32>,
    @location(4) uv_rect: vec4<f32>,
    @location(5) param: vec4<f32>,
};
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) wpos: vec3<f32>,
    @location(1) nrm: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) tint: vec4<f32>,
    @location(4) @interpolate(flat) uv_rect: vec4<f32>,
    @location(5) @interpolate(flat) param: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = u.view_proj * vec4<f32>(v.pos, 1.0);
    o.wpos = v.pos;
    o.nrm = v.nrm;
    o.uv = v.uv;
    o.tint = v.tint;
    o.uv_rect = v.uv_rect;
    o.param = v.param;
    return o;
}

@fragment
fn fs_main(i: VOut) -> @location(0) vec4<f32> {
    let uv = i.uv_rect.xy + i.uv * i.uv_rect.zw;
    let tex_col = textureSample(tex, samp, uv);

    let col = tex_col * i.tint;
    let alpha = col.a * i.param.y;
    let dist = distance(i.wpos, u.cam_pos.xyz);
    if (i.param.x > 0.5) {
        let d = dist * u.fog.w * 0.35;
        let fog_k = 1.0 - exp(-d * d);
        return vec4<f32>(mix(col.rgb, u.fog.rgb, fog_k), alpha);
    }
    let n = normalize(i.nrm);
    let sun_dir = normalize(u.sun.xyz);
    let ndl = max(dot(n, sun_dir), 0.0);
    var light = vec3<f32>(0.38, 0.41, 0.44) * u.sun_color.w;
    light = light + u.sun_color.rgb * (ndl * 0.55 + ndl * ndl * 0.25);
    var point_sum = vec3<f32>(0.0, 0.0, 0.0);
    for (var li: i32 = 0; li < 16; li = li + 1) {
        if (li >= i32(u.info.y)) { break; }
        let lp = u.lights[li];
        let to_l = lp.xyz - i.wpos;
        let d = length(to_l);
        let dir_l = select(to_l / max(d, 0.001), vec3<f32>(0.0, 1.0, 0.0), d < 0.001);
        let att = clamp(1.0 - d / lp.w, 0.0, 1.0);
        let ndl_l = max(dot(n, dir_l), 0.0) * 0.55 + 0.45;
        point_sum = point_sum + u.light_colors[li].rgb * u.light_colors[li].w * att * att * att * ndl_l;
    }
    light = light + min(point_sum, vec3<f32>(1.1, 1.1, 1.1));
    // garde-fou NaN (NaN != NaN) : tout éclairage invalide devient neutre
    if (light.x != light.x || light.y != light.y || light.z != light.z) {
        light = vec3<f32>(0.5, 0.5, 0.5);
    }
    var lit = col.rgb * light;
    lit = lit / (lit + vec3<f32>(1.0)); // tone mapping Reinhard (anti-surexposition)
    let fd2 = dist * u.fog.w;
    let fog_k = 1.0 - exp(-fd2 * fd2);
    lit = mix(lit, u.fog.rgb, fog_k);
    return vec4<f32>(lit, alpha);
}
"#;

const FX_WGSL: &str = r#"
struct FxUniforms {
    view_proj: mat4x4<f32>,
    cam_right: vec4<f32>,
    cam_up: vec4<f32>,
};
struct FxVIn {
    @location(0) corner: vec2<f32>,
    @location(1) center_size: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv_rect: vec4<f32>,
};
struct FxVOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) layer: f32,
};
@group(0) @binding(0) var<uniform> u: FxUniforms;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

@vertex
fn vs_main(v: FxVIn) -> FxVOut {
    var o: FxVOut;
    let world = v.center_size.xyz + (u.cam_right.xyz * v.corner.x + u.cam_up.xyz * v.corner.y) * v.center_size.w;
    o.clip = u.view_proj * vec4<f32>(world, 1.0);
    o.uv = v.uv_rect.xy + (v.corner * 0.5 + vec2<f32>(0.5, 0.5)) * v.uv_rect.zw;
    o.color = v.color;
    o.layer = v.uv_rect.z;
    return o;
}

@fragment
fn fs_main(i: FxVOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, i.uv);
    return vec4<f32>(i.color.rgb, c.a * i.color.a);
}
"#;

const UI_WGSL: &str = r#"
struct UiUniforms {
    ortho: mat4x4<f32>,
};
struct UiVIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};
struct UiVOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: UiUniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@vertex
fn vs_main(v: UiVIn) -> UiVOut {
    var o: UiVOut;
    o.clip = u.ortho * vec4<f32>(v.pos, 0.0, 1.0);
    o.uv = v.uv;
    o.color = v.color;
    return o;
}

@fragment
fn fs_main(i: UiVOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, i.uv);
    return c * i.color;
}
"#;

/// Données d'une frame à dessiner, produites par le jeu.
pub struct FrameDraw {
    pub dyn_verts: Vec<DynVertex>,
    pub fx_alpha: Vec<FxInstance>,
    pub fx_add: Vec<FxInstance>,
    pub ui_verts: Vec<UiVertex>,
    pub cam: Camera,
    pub fog_color: Vec3,
    pub fog_density: f32,
    /// (pos, rayon, couleur, intensité)
    pub lights: Vec<(Vec3, f32, Vec3, f32)>,
    pub time: f32,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const MAX_DYN_VERTS: usize = 200_000;
const MAX_FX: usize = 50_000;
const MAX_UI: usize = 150_000;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    depth_view: wgpu::TextureView,
    world_pipeline: wgpu::RenderPipeline,
    fx_alpha_pipeline: wgpu::RenderPipeline,
    fx_add_pipeline: wgpu::RenderPipeline,
    ui_pipeline: wgpu::RenderPipeline,
    world_bind: wgpu::BindGroup,
    fx_bind0: wgpu::BindGroup,
    fx_bind1: wgpu::BindGroup,
    ui_bind: wgpu::BindGroup,
    fx_uniform: wgpu::Buffer,
    ui_uniform: wgpu::Buffer,
    world_uniform: wgpu::Buffer,
    static_buffer: wgpu::Buffer,
    static_count: u32,
    dyn_buffer: wgpu::Buffer,
    corner_buffer: wgpu::Buffer,
    fx_alpha_buffer: wgpu::Buffer,
    fx_add_buffer: wgpu::Buffer,
    ui_buffer: wgpu::Buffer,
    ui_texture: wgpu::Texture,
    world_texture: wgpu::Texture,
}

struct Tex {
    view: wgpu::TextureView,
}

impl Renderer {
    pub fn new(window: std::sync::Arc<winit::window::Window>, static_verts: &[DynVertex], ui_atlas_px: &[u8], world_atlas_px: &[u8]) -> Self {
        let size = window.inner_size();
        // MCD_BACKEND=vulkan|gl force un backend (QA headless sur lavapipe) ; sinon tous.
        let backends = match std::env::var("MCD_BACKEND").as_deref() {
            Ok("vulkan") => wgpu::Backends::VULKAN,
            Ok("gl") => wgpu::Backends::GL,
            _ => wgpu::Backends::all(),
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends, ..Default::default() });
        let surface = instance
            .create_surface(window)
            .expect("création de surface impossible");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("aucun adaptateur GPU (essayez xvfb-run avec Mesa)");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("création du périphérique impossible");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let size_out = (config.width, config.height);
        let depth_view = Self::create_depth(&device, config.width, config.height);

        let (world_tex, world_tex_view) = Self::make_texture_mips(&device, &queue, world_atlas_px, crate::atlas::ATLAS_SIZE as u32, crate::atlas::MIP_COUNT as u32 + 1);
        let (ui_texture, ui_tex_view) = Self::make_texture(&device, &queue, ui_atlas_px, crate::ui_atlas::UI_W as u32, crate::ui_atlas::UI_H as u32);
        let world_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let ui_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // ---- Layouts de bind groups ----
        let make_tex_bgl = |device: &wgpu::Device| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
        };
        let world_bgl = make_tex_bgl(&device);
        let ui_bgl = make_tex_bgl(&device);
        let fx_bgl0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        let fx_bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // ---- Uniforms + bind groups ----
        let world_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 768,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fx_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ui_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let world_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &world_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: world_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&world_tex_view.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&world_sampler) },
            ],
        });
        let fx_bind0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &fx_bgl0,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: fx_uniform.as_entire_binding() }],
        });
        let fx_bind1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &fx_bgl1,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&world_tex_view.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&world_sampler) },
            ],
        });
        let ui_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &ui_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ui_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&ui_tex_view.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&ui_sampler) },
            ],
        });

        // ---- Pipelines ----
        let world_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(WORLD_WGSL.into()),
        });
        let dyn_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DynVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 3 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 48, shader_location: 4 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 64, shader_location: 5 },
            ],
        };
        let world_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&world_bgl],
            push_constant_ranges: &[],
        });
        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&world_layout),
            vertex: wgpu::VertexState {
                module: &world_module,
                entry_point: "vs_main",
                buffers: &[dyn_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let fx_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(FX_WGSL.into()),
        });
        let corner_layout = wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };
        let fx_inst_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FxInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 2 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 3 },
            ],
        };
        let fx_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&fx_bgl0, &fx_bgl1],
            push_constant_ranges: &[],
        });
        let add_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let make_fx_pipeline = |blend: wgpu::BlendState| -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&fx_layout),
                vertex: wgpu::VertexState {
                    module: &fx_module,
                    entry_point: "vs_main",
                    buffers: &[corner_layout.clone(), fx_inst_layout.clone()],
                    },
                fragment: Some(wgpu::FragmentState {
                    module: &fx_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                })
        };
        let fx_alpha_pipeline = make_fx_pipeline(wgpu::BlendState::ALPHA_BLENDING);
        let fx_add_pipeline = make_fx_pipeline(add_blend);

        let ui_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(UI_WGSL.into()),
        });
        let ui_layout_vb = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 8, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 2 },
            ],
        };
        let ui_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&ui_bgl],
            push_constant_ranges: &[],
        });
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&ui_pipe_layout),
            vertex: wgpu::VertexState {
                module: &ui_module,
                entry_point: "vs_main",
                buffers: &[ui_layout_vb],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        // ---- Buffers ----
        let static_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(static_verts),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
        });
        let vertex_buf = |size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let dyn_buffer = vertex_buf(MAX_DYN_VERTS * std::mem::size_of::<DynVertex>());
        let fx_alpha_buffer = vertex_buf(MAX_FX * std::mem::size_of::<FxInstance>());
        let fx_add_buffer = vertex_buf(MAX_FX * std::mem::size_of::<FxInstance>());
        let ui_buffer = vertex_buf(MAX_UI * std::mem::size_of::<UiVertex>());
        let corner_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[-1.0f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Renderer {
            device,
            queue,
            surface,
            config,
            size: size_out,
            depth_view,
            world_pipeline,
            fx_alpha_pipeline,
            fx_add_pipeline,
            ui_pipeline,
            world_bind,
            fx_bind0,
            fx_bind1,
            ui_bind,
            fx_uniform,
            ui_uniform,
            world_uniform,
            static_buffer,
            static_count: static_verts.len() as u32,
            dyn_buffer,
            corner_buffer,
            fx_alpha_buffer,
            fx_add_buffer,
            ui_buffer,
            ui_texture,
            world_texture: world_tex,
        }
    }

    /// Recharge la texture UI (nouveaux glyphes de texte).
    pub fn update_ui_atlas(&mut self, pixels: &[u8]) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.ui_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(crate::ui_atlas::UI_W as u32 * 4),
                rows_per_image: Some(crate::ui_atlas::UI_H as u32),
            },
            wgpu::Extent3d {
                width: crate::ui_atlas::UI_W as u32,
                height: crate::ui_atlas::UI_H as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    fn create_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        tex.create_view(&wgpu::TextureViewDescriptor::default())
    }


    /// Atlas 2D avec mips CPU consécutifs (padding 256 pour les petits niveaux).
    fn make_texture_mips(device: &wgpu::Device, queue: &wgpu::Queue, rgba: &[u8], w: u32, mips: u32) -> (wgpu::Texture, Tex) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: w, depth_or_array_layers: 1 },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        for mip in 0..mips {
            let lw = w >> mip;
            let layer_px = (lw * lw * 4) as usize;
            let base = crate::atlas::mip_offset(mip as usize);
            let level = &rgba[base..base + layer_px];
            let row = lw as usize * 4;
            let (padded, bpr) = if row.is_multiple_of(256) {
                (level.to_vec(), row as u32)
            } else {
                let stride = row.div_ceil(256) * 256;
                let mut buf = vec![0u8; stride * lw as usize];
                for y in 0..lw as usize {
                    let src = y * row;
                    buf[y * stride..y * stride + row].copy_from_slice(&level[src..src + row]);
                }
                (buf, stride as u32)
            };
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &tex,
                    mip_level: mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &padded,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(lw),
                },
                wgpu::Extent3d { width: lw, height: lw, depth_or_array_layers: 1 },
            );
        }
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, Tex { view })
    }

    /// Texture RGBA simple (interface).
    fn make_texture(device: &wgpu::Device, queue: &wgpu::Queue, rgba: &[u8], w: u32, h: u32) -> (wgpu::Texture, Tex) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, Tex { view })
    }

    /// Debug : lit le mip0 de la texture monde et le sauvegarde en PNG.
    pub fn dump_world_texture(&self, path: &str) {
        let w = crate::atlas::ATLAS_SIZE as u32;
        let bytes_per_row = w * 4;
        let buf_size = (bytes_per_row * w) as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buf_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.world_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(w),
                },
            },
            wgpu::Extent3d { width: w, height: w, depth_or_array_layers: 1 },
        );
        self.queue.submit([enc.finish()]);
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().ok().unwrap().unwrap();
        let data = slice.get_mapped_range();
        let mut rgba = data.to_vec();
        drop(data);
        buffer.unmap();
        // BGRA -> RGBA puis suppression de l'alpha pour le PNG
        for px in rgba.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
        let img = image::RgbaImage::from_raw(w, w, rgba).unwrap();
        img.save(path).ok();
    }

    /// Debug : copie le vertex buffer static vers un fichier pour comparaison CPU/GPU.
    pub fn dump_static_buffer(&self, path: &str, byte_len: u64) {
        let out = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.copy_buffer_to_buffer(&self.static_buffer, 0, &out, 0, byte_len);
        self.queue.submit([enc.finish()]);
        let slice = out.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().ok().unwrap().unwrap();
        let data = slice.get_mapped_range();
        std::fs::write(path, &data[..]).ok();
        drop(data);
        out.unmap();
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = Self::create_depth(&self.device, w, h);
        self.size = (w, h);
    }

    /// Dessine la frame. Si `capture`, retourne l'image lue depuis la surface.
    pub fn render(&mut self, fd: &FrameDraw, capture: bool) -> Option<image::RgbaImage> {
        let (w, h) = self.size;
        if w == 0 || h == 0 {
            return None;
        }
        // ---- Uniforms monde ----
        let mut ub = [0u8; 768];
        let vp = fd.cam.view_proj();
        ub[0..64].copy_from_slice(bytemuck::cast_slice(&vp.to_cols_array_2d()));
        let eye = [fd.cam.eye.x, fd.cam.eye.y, fd.cam.eye.z, 1.0f32];
        let fog = [
            srgb_to_linear(fd.fog_color.x),
            srgb_to_linear(fd.fog_color.y),
            srgb_to_linear(fd.fog_color.z),
            fd.fog_density,
        ];
        let sun = [-0.4f32, -0.85, -0.3, 1.0];
        let sun_color = [srgb_to_linear(0.38), srgb_to_linear(0.44), srgb_to_linear(0.5), 1.0];
        let info = [fd.time, fd.lights.len().min(16) as f32, 0.0, 0.0];
        let mut off = 64usize;
        for arr in [&eye, &fog, &sun, &sun_color, &info] {
            ub[off..off + 16].copy_from_slice(bytemuck::bytes_of(arr));
            off += 16;
        }
        for (i, (pos, radius, color, intensity)) in fd.lights.iter().take(16).enumerate() {
            let base = 144 + i * 16;
            ub[base..base + 16].copy_from_slice(bytemuck::cast_slice(&[pos.x, pos.y, pos.z, *radius]));
            let base2 = 144 + 256 + i * 16;
            ub[base2..base2 + 16].copy_from_slice(bytemuck::cast_slice(&[
                srgb_to_linear(color.x),
                srgb_to_linear(color.y),
                srgb_to_linear(color.z),
                *intensity,
            ]));
        }
        self.queue.write_buffer(&self.world_uniform, 0, &ub);

        // ---- Uniforms UI (ortho pixels) ----
        let ortho = Mat4::orthographic_rh(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
        self.queue.write_buffer(&self.ui_uniform, 0, bytemuck::cast_slice(&ortho.to_cols_array_2d()));

        // ---- Uniforms FX ----
        let fwd = (fd.cam.target - fd.cam.eye).normalize_or_zero();
        let right = fwd.cross(crate::camera::UP).normalize_or_zero();
        let up = right.cross(fwd).normalize_or_zero();
        let mut fxu = [0u8; 128];
        fxu[0..64].copy_from_slice(bytemuck::cast_slice(&vp.to_cols_array_2d()));
        fxu[64..80].copy_from_slice(bytemuck::cast_slice(&[right.x, right.y, right.z, 0.0f32]));
        fxu[80..96].copy_from_slice(bytemuck::cast_slice(&[up.x, up.y, up.z, 0.0f32]));
        self.queue.write_buffer(&self.fx_uniform, 0, &fxu);

        // ---- Vertex dynamiques ----
        let n_dyn = fd.dyn_verts.len().min(MAX_DYN_VERTS);
        if n_dyn > 0 {
            self.queue.write_buffer(&self.dyn_buffer, 0, bytemuck::cast_slice(&fd.dyn_verts[..n_dyn]));
        }
        let n_fx_a = fd.fx_alpha.len().min(MAX_FX);
        if n_fx_a > 0 {
            self.queue.write_buffer(&self.fx_alpha_buffer, 0, bytemuck::cast_slice(&fd.fx_alpha[..n_fx_a]));
        }
        let n_fx_s = fd.fx_add.len().min(MAX_FX);
        if n_fx_s > 0 {
            self.queue.write_buffer(&self.fx_add_buffer, 0, bytemuck::cast_slice(&fd.fx_add[..n_fx_s]));
        }
        let n_ui = fd.ui_verts.len().min(MAX_UI);
        if n_ui > 0 {
            self.queue.write_buffer(&self.ui_buffer, 0, bytemuck::cast_slice(&fd.ui_verts[..n_ui]));
        }

        // ---- Passes ----
        let surface_tex = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return None,
        };
        let view = surface_tex.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let clear = wgpu::Color {
            r: fd.fog_color.x as f64,
            g: fd.fog_color.y as f64,
            b: fd.fog_color.z as f64,
            a: 1.0,
        };
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.world_pipeline);
            rpass.set_bind_group(0, &self.world_bind, &[]);
            rpass.set_vertex_buffer(0, self.static_buffer.slice(..));
            // Dessin par tranches (contourne un bug de rasterisation en grand draw
            // sur certains rasteriseurs logiciels ; harmless sur GPU matériel).
            let chunk = 3_072u32; // multiple de 6 (quads entiers)
            let mut start = 0u32;
            while start < self.static_count {
                let end = (start + chunk).min(self.static_count);
                let len = end - start;
                let len = len - len % 6;
                if len > 0 {
                    rpass.draw(start..start + len, 0..1);
                }
                start += len;
            }
            if n_dyn > 0 {
                rpass.set_vertex_buffer(0, self.dyn_buffer.slice(..));
                rpass.draw(0..n_dyn as u32, 0..1);
            }
            if n_fx_a > 0 {
                rpass.set_pipeline(&self.fx_alpha_pipeline);
                rpass.set_bind_group(0, &self.fx_bind0, &[]);
                rpass.set_bind_group(1, &self.fx_bind1, &[]);
                rpass.set_vertex_buffer(0, self.corner_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.fx_alpha_buffer.slice(..));
                rpass.draw(0..6, 0..n_fx_a as u32);
            }
            if n_fx_s > 0 {
                rpass.set_pipeline(&self.fx_add_pipeline);
                rpass.set_bind_group(0, &self.fx_bind0, &[]);
                rpass.set_bind_group(1, &self.fx_bind1, &[]);
                rpass.set_vertex_buffer(0, self.corner_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.fx_add_buffer.slice(..));
                rpass.draw(0..6, 0..n_fx_s as u32);
            }
        }
        if n_ui > 0 {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.ui_pipeline);
            rpass.set_bind_group(0, &self.ui_bind, &[]);
            rpass.set_vertex_buffer(0, self.ui_buffer.slice(..));
            rpass.draw(0..n_ui as u32, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        let captured = if capture { self.read_surface(&surface_tex.texture) } else { None };
        surface_tex.present();
        captured
    }

    /// Lit la texture de la surface et la convertit en image RGBA.
    fn read_surface(&mut self, texture: &wgpu::Texture) -> Option<image::RgbaImage> {
        let (w, h) = self.size;
                let bpp = 4u32;
        let bytes_per_row = (w * bpp).div_ceil(256) * 256;
        let buf_size = bytes_per_row * h;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buf_size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().ok()?
            .expect("mapping échoué");
        let data = slice.get_mapped_range();
        let mut img = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            let start = (y * bytes_per_row) as usize;
            let row = &data[start..start + (w * 4) as usize];
            for px in row.as_chunks::<4>().0 {
                // la surface est BGRA : on remet en RGBA
                img.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            }
        }
        drop(data);
        buffer.unmap();
        image::RgbaImage::from_raw(w, h, img)
    }
}

/// Conversion sRGB -> linéaire (les couleurs d'ambiance sont choisies « à l'œil »).
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

