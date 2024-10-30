use gl::types::*;
use glutin::display::GlDisplay;
use image::ImageReader;
use log::{error, info};
use shader::{Shader, ShaderType};
use std::ffi::{c_void, CStr, CString};
use std::os::raw;

mod shader;

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

pub struct Renderer {
    vertex_shader: Shader,
    fragment_shader: Shader,
    program: GLuint,
    vertex_array_object: GLuint,
    per_frame_buffer_object: GLuint,
    texture_id: GLuint,
    draw_config: DrawConfig,
}

#[repr(C)]
struct PerFrameData {
    perspective_transform: [f32; 16],
    wire_frame_enabled: u32,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
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
        let program = create_program(vertex_shader.handle, fragment_shader.handle).unwrap();

        let texture_id = create_texture("textures/stone.png").unwrap();

        let vertex_array_object = create_vertex_array_object().unwrap();
        let per_frame_buffer_object = create_vertex_buffer_object(size_of::<PerFrameData>()).unwrap();

        unsafe {
            gl::UseProgram(program);
            gl::BindVertexArray(vertex_array_object);

            gl::BindTextures( 0, 1, &texture_id);

            gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, per_frame_buffer_object, 0, size_of::<PerFrameData>() as isize);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::POLYGON_OFFSET_LINE);
            gl::PolygonOffset(-1.0, -1.0);
        }

        let draw_config = DrawConfig::new((300, 300));

        Self {
            texture_id,
            vertex_shader,
            fragment_shader,
            program,
            vertex_array_object,
            per_frame_buffer_object,
            draw_config,
        }
    }

    pub fn draw(&mut self, delta: f32, frame_delta: f32) {
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
            //println!("{}", translation_matrix);
            let translation_matrix_slice = translation_matrix.as_slice();

            let mut per_frame_date = PerFrameData {
                perspective_transform: translation_matrix_slice.try_into().expect("slice is incorrect length"),
                wire_frame_enabled: 0,
            };

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            let per_frame_data_ptr: *mut c_void = &mut per_frame_date as *mut _ as *mut c_void;
            gl::NamedBufferSubData(self.per_frame_buffer_object, 0, size_of::<PerFrameData>() as isize, per_frame_data_ptr);

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            per_frame_date.wire_frame_enabled = 1;
            gl::NamedBufferSubData(self.per_frame_buffer_object, 0, size_of::<PerFrameData>() as isize, per_frame_data_ptr);

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
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

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, & self.texture_id);
            //delete shaders
            gl::DeleteProgram(self.program);

            gl::DeleteVertexArrays(1, &self.vertex_array_object);
        }
    }
}

fn get_gl_string(variant: GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

fn create_vertex_array_object() -> Result<GLuint, String> {
    let vao = unsafe {
        let mut vao: GLuint = 0;
        gl::CreateVertexArrays(1, &mut vao);
        vao
    };
    return Ok(vao);
}

fn create_vertex_buffer_object(memory_size: usize) -> Result<GLuint, String> {
    let vertex_buffer_object = unsafe {
        let mut vbo: GLuint = 0;
        gl::CreateBuffers(1, &mut vbo);

        gl::NamedBufferStorage(vbo, memory_size as isize, std::ptr::null(), gl::DYNAMIC_STORAGE_BIT);
        vbo
    };
    return Ok(vertex_buffer_object);
}

fn create_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
    let program_id = unsafe {
        let program_id = gl::CreateProgram();
        if program_id == 0 {
            return Err("glCreateProgram failed".to_string());
        };

        gl::AttachShader(program_id, vertex_shader);
        gl::AttachShader(program_id, fragment_shader);

        gl::LinkProgram(program_id);

        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut log_len = 0;
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_len);
            let mut log_buf: Vec<u8> = Vec::with_capacity(log_len as usize);
            log_buf.set_len((log_len as usize) - 1);
            gl::GetProgramInfoLog(program_id, log_len, std::ptr::null_mut(), log_buf.as_mut_ptr() as *mut GLchar);

            return Err(String::from_utf8(log_buf).unwrap().to_string());
        }
        program_id
    };

    return Ok(program_id);
}

fn create_texture(source_file: &str) -> Result<GLuint, String> {
    let texture_id = unsafe {

        let img = match ImageReader::open(source_file) {
            Ok(img) => img,
            Err(e) => return Err(String::from("error opening image file")),
        };
        let img = match img.decode() {
            Ok(img) => img,
            Err(e) => return Err(String::from("error decoding image")),
        };

        let img = img.into_rgb8();

        let width: i32 = img.width() as i32;
        let height: i32 = img.height() as i32;

        let mut tex: GLuint = 0;
        // create texture 2D
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut tex);
        gl::TextureParameteri(tex, gl::TEXTURE_MAX_LEVEL, 0);
        // these as statements are suspect
        gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TextureStorage2D(tex, 1, gl::RGB8, width, height);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TextureSubImage2D(tex, 0, 0, 0, width, height, gl::RGB, gl::UNSIGNED_BYTE, (&img as &[u8]).as_ptr() as *const c_void);

        tex
    };
    return Ok(texture_id);
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

    error!("{:?} [{}] {}: {} {}", id, severity, er_type, source, message);
}
