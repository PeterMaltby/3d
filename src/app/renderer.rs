use gl::types::*;
use glutin::display::GlDisplay;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Shader {
    id: GLuint,
    shader_type: GLenum,
    source_file: String,
}

pub struct Program {
    id: GLuint,
    shader_ids: Vec<GLuint>,
}

pub struct VertexAttribute {
    component_size: GLuint,
    stride: u32,
    offset: u32,
}

pub struct Renderer {
    gl: gl::Gl,
    vertex_array_objects: Vec<GLuint>,
    vertex_buffer_objects: Vec<GLuint>,
    shaders: HashMap<GLuint, Shader>,
    programs: HashMap<GLuint, Program>,
    runners: Vec<Runner>,
}

pub struct Runner {
    program: GLuint,
    vertex_array_object: GLuint,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        let gl = gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
            println!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(&gl, gl::VERSION) {
            println!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
            println!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        Self {
            runners: Vec::new(),
            programs: HashMap::new(),
            shaders: HashMap::new(),
            vertex_array_objects: Vec::new(),
            vertex_buffer_objects: Vec::new(),
            gl,
        }
    }

    pub fn init(&mut self) {
        self.init_error_callback(true, true);

        let vertex_shader = self.create_shader(gl::VERTEX_SHADER, "shaders/vertex.glsl").unwrap();
        let fragment_shader = self.create_shader(gl::FRAGMENT_SHADER, "shaders/fragment.glsl").unwrap();

        let program = self.create_program(vertex_shader, fragment_shader).unwrap();

        let vertex_array_object = self.create_vertex_array_object().unwrap();

        self.runners.push(Runner { program, vertex_array_object });
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 0.9)
    }

    pub fn draw_with_clear_color(&self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe {
            self.gl.ClearColor(red, green, blue, alpha);

            //self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            for runner in &self.runners {
                self.gl.UseProgram(runner.program);
                self.gl.BindVertexArray(runner.vertex_array_object);

                self.gl.Clear(gl::COLOR_BUFFER_BIT);
                self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
            }
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    fn init_error_callback(&self, sync: bool, debug: bool) {
        extern "system" fn gl_debug_callback(
            source: GLenum,
            er_type: GLenum,
            id: GLuint,
            severity: GLenum,
            _: GLsizei,
            message: *const GLchar,
            _: *mut raw::c_void,
        ) {
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

            println!("{:?} [{}] {}: {} {}", id, severity, er_type, source, message);
        }

        unsafe {
            if sync {
                self.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            }
            self.gl.DebugMessageCallback(Some(gl_debug_callback), 0 as *const _);
            if debug {
                self.gl.Enable(gl::DEBUG_OUTPUT);
            }
        }
    }

    fn create_vertex_array_object(&mut self) -> Result<GLuint, String> {
        let vao = unsafe {
            let mut vao: GLuint = 0;
            self.gl.CreateVertexArrays(1, &mut vao);
            vao
        };
        return Ok(vao);
    }

    fn create_shader(&mut self, shader_type: GLenum, source_file: &str) -> Result<GLuint, String> {
        let shader = unsafe {
            let shader = self.gl.CreateShader(shader_type);

            let shader_code = match fs::read_to_string(source_file) {
                Ok(f) => f,
                Err(e) => return Err(format!("unable to read file \"{}\" {}", source_file, e)),
            };

            let source_c_str = CString::new(shader_code.as_bytes()).unwrap();
            self.gl.ShaderSource(shader, 1, &(source_c_str.as_ptr()), &(shader_code.len() as GLint));
            self.gl.CompileShader(shader);

            let mut status = gl::FALSE as GLint;
            self.gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                self.gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log_buf: Vec<u8> = Vec::with_capacity(log_len as usize);
                log_buf.set_len((log_len as usize) - 1);
                self.gl
                    .GetShaderInfoLog(shader, log_len, std::ptr::null_mut(), log_buf.as_mut_ptr() as *mut GLchar);

                return Err(String::from_utf8(log_buf).unwrap().to_string());
            }

            shader
        };

        self.shaders.insert(
            shader,
            Shader {
                id: shader,
                shader_type,
                source_file: source_file.to_string(),
            },
        );
        println!("created shader {}: {}", shader, source_file);

        Ok(shader)
    }

    fn create_program(&mut self, vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
        let program_id = unsafe {
            let program_id = self.gl.CreateProgram();
            if program_id == 0 {
                return Err("glCreateProgram failed".to_string());
            };

            self.gl.AttachShader(program_id, vertex_shader);
            self.gl.AttachShader(program_id, fragment_shader);

            self.gl.LinkProgram(program_id);

            let mut status = gl::FALSE as GLint;
            self.gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                self.gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log_buf: Vec<u8> = Vec::with_capacity(log_len as usize);
                log_buf.set_len((log_len as usize) - 1);
                self.gl
                    .GetProgramInfoLog(program_id, log_len, std::ptr::null_mut(), log_buf.as_mut_ptr() as *mut GLchar);

                return Err(String::from_utf8(log_buf).unwrap().to_string());
            }
            program_id
        };

        self.programs.insert(
            program_id,
            Program {
                id: program_id,
                shader_ids: vec![vertex_shader, fragment_shader],
            },
        );

        return Ok(program_id);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for (shader_id, shader) in &self.shaders {
                self.gl.DeleteShader(*shader_id);
                println!("deleted shader {}: {}", shader_id, shader.source_file);
            }
            for (program_id, program) in &self.programs {
                self.gl.DeleteProgram(*program_id);
                println!("deleted program {}: {:?}", program_id, program.shader_ids);
            }

            //self.gl.DeleteBuffers(1, &self.vbo);

            for vertex_array_object in &self.vertex_array_objects {
                self.gl.DeleteVertexArrays(1, vertex_array_object);
            }
        }
    }
}

fn get_gl_string(gl: &gl::Gl, variant: GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
     0.0,  0.5,  0.0,  1.0,  0.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];
