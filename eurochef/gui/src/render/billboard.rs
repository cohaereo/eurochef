use anyhow::Result;
use glam::{Mat4, Vec3};
use glow::HasContext;

use super::{
    blend::set_blending_mode,
    pickbuffer::{PickBuffer, PickBufferType},
    viewer::RenderContext,
};

pub struct BillboardRenderer {
    quad: glow::VertexArray,
}

impl BillboardRenderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        Ok(Self {
            quad: Self::quad_vao(gl),
        })
    }

    const VERTEX_DATA: &'static [[f32; 5]] = &[
        [-0.5, -0.5, 0.0, 0.0, 1.0],
        [-0.5, 0.5, 0.0, 0.0, 0.0],
        [0.5, -0.5, 0.0, 1.0, 1.0],
        [0.5, 0.5, 0.0, 1.0, 0.0],
    ];
    fn quad_vao(gl: &glow::Context) -> glow::VertexArray {
        unsafe {
            let vertex_array = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vertex_array));
            let vertex_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(Self::VERTEX_DATA),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * 4, 0);

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                5 * 4,
                3 * std::mem::size_of::<f32>() as i32,
            );

            vertex_array
        }
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        context: &RenderContext,
        texture: glow::Texture,
        pos: Vec3,
        scale: f32,
    ) {
        set_blending_mode(gl, super::blend::BlendMode::Cutout);
        unsafe {
            gl.use_program(Some(context.shaders.sprite3d));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(context.shaders.sprite3d, "u_view")
                    .as_ref(),
                false,
                &context.uniforms.view.to_cols_array(),
            );

            let model = Mat4::from_translation(pos)
                * Mat4::from_quat(-context.uniforms.camera_rotation)
                * Mat4::from_scale(Vec3::splat(scale));
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(context.shaders.sprite3d, "u_model")
                    .as_ref(),
                false,
                &model.to_cols_array(),
            );

            gl.uniform_1_i32(
                gl.get_uniform_location(context.shaders.sprite3d, "u_texture")
                    .as_ref(),
                0,
            );

            gl.bind_vertex_array(Some(self.quad));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }

    pub fn render_pickbuffer(
        &self,
        gl: &glow::Context,
        context: &RenderContext,
        pos: Vec3,
        scale: f32,
        id: (PickBufferType, u32),
        pb: &PickBuffer,
    ) {
        let model = Mat4::from_translation(pos)
            * Mat4::from_quat(-context.uniforms.camera_rotation)
            * Mat4::from_scale(Vec3::splat(scale));

        pb.draw(context, gl, model, id, |gl| unsafe {
            gl.bind_vertex_array(Some(self.quad));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        });
    }
}
