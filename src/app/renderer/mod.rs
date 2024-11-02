use gl::types::*;
use anyhow::Result;
use glutin::display::GlDisplay;
use log::{error, info};
use std::os::raw;
use std::ffi::{c_void, CStr, CString};

mod shader;
use shader::{Shader, ShaderType};
mod program;
use program::Program;
mod vertex_array_objects;
use vertex_array_objects::VertexArrayObjects;
mod texture;
use texture::Texture;
mod vertex_buffer_objects;
use vertex_buffer_objects::VertexBufferObjects;

pub mod gl;

pub struct DrawConfig {
    field_of_view: f32,
    near_clipping_plane: f32,
    far_clipping_plane: f32,
    display_dimensions: (i32, i32),
    display_aspect: f32,
}

impl DrawConfig {
    fn new((width, height): (i32, i32)) -> Self {
        return DrawConfig {
            field_of_view: 45.0,
            near_clipping_plane: 0.1,
            far_clipping_plane: 1000.0,
            display_dimensions: (width, height),
            display_aspect: width as f32 / height as f32,
        };
    }
}


#[allow(unused)]
pub struct Renderer {
    vertex_shader: Shader,
    fragment_shader: Shader,
    program: Program,
    vertex_array_object: VertexArrayObjects,
    per_frame_buffer_object: VertexBufferObjects<PerFrameData>,
    texture: Texture,
    draw_config: DrawConfig,
}

#[repr(C)]
struct PerFrameData {
    perspective_transform: [f32; 16],
    wire_frame_enabled: u32,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Result<Self> {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if let Some(renderer) = get_gl_string(gl::RENDERER) {
            info!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(gl::VERSION) {
            info!("OpenGL Version {}", version.to_string_lossy());
        }
        if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
            info!("Shaders version on {}\n", shaders_version.to_string_lossy());
        }

        #[cfg(debug_assertions)]
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_callback), 0 as *const _);
            gl::Enable(gl::DEBUG_OUTPUT);
        }

        let vertex_shader = Shader::new(ShaderType::VERTEX, "shaders/vertex_tex.glsl").unwrap();
        let fragment_shader = Shader::new(ShaderType::FRAGMENT, "shaders/fragment_tex.glsl").unwrap();
        let program = Program::new(&vertex_shader, &fragment_shader)?;

        let texture = Texture::new("textures/stone.png").unwrap();

        let vertex_array_object = VertexArrayObjects::new().unwrap();
        let per_frame_buffer_object = VertexBufferObjects::new().unwrap();

        unsafe {
            program.use_program();

            vertex_array_object.bind();
    
            texture.bind();

            per_frame_buffer_object.bind();

            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::POLYGON_OFFSET_LINE);
            gl::PolygonOffset(-1.0, -1.0);
        }

        let draw_config = DrawConfig::new((300, 300));

        Ok(Self {
            texture,
            vertex_shader,
            fragment_shader,
            program,
            vertex_array_object,
            per_frame_buffer_object,
            draw_config,
        })
    }

    pub fn draw(&mut self, delta: f32, _frame_delta: f32) {
        unsafe {
            let identity_matrix = glm::Mat4::identity();
            let translation_vector = glm::vec3(0.0, 0.0, -3.5);
            let translation_matrix = glm::translate(&identity_matrix, &translation_vector);
            let rotation_vec = glm::vec3(1.0, 1.0, 1.0);

            let translation_matrix = glm::rotate(&translation_matrix, delta, &rotation_vec);

            let perspective_matrix = glm::perspective(
                self.draw_config.display_aspect,
                self.draw_config.field_of_view,
                self.draw_config.near_clipping_plane,
                self.draw_config.far_clipping_plane,
            );

            let translation_matrix = perspective_matrix * translation_matrix;
            let translation_matrix_slice = translation_matrix.as_slice();

            let per_frame_date = PerFrameData {
                perspective_transform: translation_matrix_slice.try_into().expect("slice is incorrect length"),
                wire_frame_enabled: 0,
            };

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            self.per_frame_buffer_object.sub_buffer(per_frame_date);

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            //gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.draw_config.display_dimensions = (width, height);
            self.draw_config.display_aspect = width as f32 / height as f32;
            gl::Viewport(0, 0, width, height);
        }
    }
}

fn get_gl_string(variant: GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

extern "system" fn gl_debug_callback(source: GLenum, er_type: GLenum, id: GLuint, severity: GLenum, _: GLsizei, message: *const GLchar, _: *mut raw::c_void) {
    let message = unsafe { String::from_utf8(CStr::from_ptr(message).to_bytes().to_vec()).unwrap() };

    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINODW SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER COMPILE",
        gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "SOURCE APP",
        gl::DEBUG_SOURCE_OTHER => "OTHER",
        _ => "UNKNOWN",
    };

    let er_type = match er_type {
        gl::DEBUG_TYPE_ERROR => "ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED BEHAVIOUR",
        gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
        gl::DEBUG_TYPE_MARKER => "MARKER",
        gl::DEBUG_TYPE_PUSH_GROUP => "PUSH GROUP",
        gl::DEBUG_TYPE_POP_GROUP => "POP GROUP",
        gl::DEBUG_TYPE_OTHER => "OTHER",
        _ => "UNKNOWN",
    };

    let severity = match severity {
        gl::DEBUG_SEVERITY_NOTIFICATION => "NOTICE",
        gl::DEBUG_SEVERITY_LOW => "LOW",
        gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
        gl::DEBUG_SEVERITY_HIGH => "HIGH",
        _ => "UNKNOWN",
    };

    error!("GL ERROR: {:?} [{}] {}: {} {}", id, severity, er_type, source, message);
}
