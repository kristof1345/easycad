use crate::Vertex;
use egui_wgpu::wgpu;
use wgpu::{BindGroupLayout, Device, ShaderModule, SurfaceConfiguration};

pub struct Pipeline {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        shader: &ShaderModule,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                // compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                // compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                conservative: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            // cache: None,
        });

        Self { render_pipeline }
    }

    pub fn new_circle_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        shader: &ShaderModule,
        camera_bind_group_layout: &BindGroupLayout,
        circle_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, circle_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline Circle"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                // compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
                // buffers: &[wgpu::VertexBufferLayout {
                //             array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                //             step_mode: wgpu::VertexStepMode::Vertex,
                //             attributes: &[
                //                 wgpu::VertexAttribute {
                //                     offset: 0,
                //                     shader_location: 0,
                //                     format: wgpu::VertexFormat::Float32x3, // position
                //                 },
                //                 wgpu::VertexAttribute {
                //                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                //                     shader_location: 1,
                //                     format: wgpu::VertexFormat::Float32x3, // color
                //                 },
                //             ],
                //         }],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                // compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                ..Default::default() // front_face: wgpu::FrontFace::Ccw,
                                     // cull_mode: Some(wgpu::Face::Back),
                                     // unclipped_depth: false,
                                     // conservative: false,
                                     // polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            // cache: None,
        });

        Self { render_pipeline }
    }
}
