use crate::wgpu_utils::pipelines::*;
use crate::wgpu_utils::shader::*;
use std::{path::Path, rc::Rc};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LineVertex {
    pub position: cgmath::Point3<f32>,
    pub color: cgmath::Vector3<f32>,
}

impl LineVertex {
    pub fn new(pos: cgmath::Point3<f32>, color: cgmath::Vector3<f32>) -> Self {
        LineVertex { position: pos, color }
    }
}

const LINE_VERTEX_SIZE: usize = std::mem::size_of::<LineVertex>();

pub struct StaticLineRenderer {
    render_pipeline: RenderPipelineHandle,
    vertex_buffer: wgpu::Buffer,

    max_num_lines: usize,
    num_lines: usize,
}

impl StaticLineRenderer {
    pub fn new(
        device: &wgpu::Device,
        shader_dir: &ShaderDirectory,
        pipeline_manager: &mut PipelineManager,
        per_frame_bind_group_layout: &wgpu::BindGroupLayout,
        max_num_lines: usize,
    ) -> Self {
        let mut render_pipeline_desc = RenderPipelineCreationDesc::new(
            Rc::new(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&per_frame_bind_group_layout],
            })),
            Path::new("lines.vert"),
            Some(Path::new("lines.frag")),
        );
        render_pipeline_desc.primitive_topology = wgpu::PrimitiveTopology::LineList;
        render_pipeline_desc.vertex_state = wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: LINE_VERTEX_SIZE as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 4 * 3,
                        shader_location: 1,
                    },
                ],
            }],
        };
        let render_pipeline = pipeline_manager.create_render_pipeline(device, shader_dir, render_pipeline_desc);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("StaticLineRenderer VertexBuffer"),
            size: (max_num_lines * LINE_VERTEX_SIZE * 2) as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        StaticLineRenderer {
            render_pipeline,
            vertex_buffer,

            max_num_lines,
            num_lines: 0,
        }
    }

    pub fn clear_lines(&mut self) {
        self.num_lines = 0;
    }

    pub fn add_lines(&mut self, lines: &[LineVertex], device: &wgpu::Device, init_encoder: &mut wgpu::CommandEncoder) {
        if lines.len() + self.num_lines > self.max_num_lines {
            error!(
                "Buffer too small to add {} lines. Containing {} right now, maximum is {}",
                lines.len(),
                self.num_lines,
                self.max_num_lines
            );
            return;
        }

        let new_vertices_size = (lines.len() * LINE_VERTEX_SIZE) as u64;
        let particle_buffer_mapping = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: Some("Buffer: StaticLine Update"),
            size: new_vertices_size,
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        unsafe {
            std::ptr::copy_nonoverlapping(
                lines.as_ptr() as *const u8,
                particle_buffer_mapping.data.as_mut_ptr(),
                new_vertices_size as usize,
            );
        }

        init_encoder.copy_buffer_to_buffer(
            &particle_buffer_mapping.finish(),
            0,
            &self.vertex_buffer,
            (self.num_lines * LINE_VERTEX_SIZE) as u64,
            new_vertices_size,
        );

        self.num_lines += lines.len();
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, pipeline_manager: &'a PipelineManager) {
        rpass.set_pipeline(pipeline_manager.get_render(&self.render_pipeline));
        let num_vertices = self.num_lines * 2;
        rpass.set_vertex_buffer(0, &self.vertex_buffer, 0, (num_vertices * LINE_VERTEX_SIZE) as u64);
        rpass.draw(0..(num_vertices as u32), 0..1);
    }
}
