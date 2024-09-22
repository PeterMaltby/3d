use gl::types::*;
use glutin::display::GlDisplay;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::fs;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Renderer {
    gl: gl::Gl,
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
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

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, "shaders/vertex.glsl");
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, "shaders/fragment.glsl");

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            // push vertex to open GL
            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);
            gl.EnableVertexAttribArray(color_attrib as GLuint);

            Self { program, vao, vbo, gl }
        }
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 0.9)
    }

    pub fn draw_with_clear_color(&self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe {
            self.gl.UseProgram(self.program);

            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.gl.ClearColor(red, green, blue, alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    pub fn init_error_callback(&self, sync: bool, debug: bool) {
        extern "system" fn gl_debug_callback(
            source: GLenum,
            er_type: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            user_param: *mut raw::c_void,
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

            println!(
                "{:?} [{}] {}: {}",
                id, severity, er_type, message
            );
        }

        unsafe {
            if sync { self.gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); }
            self.gl.DebugMessageCallback(Some(gl_debug_callback), 0 as *const _);
            if debug { self.gl.Enable(gl::DEBUG_OUTPUT); }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(gl: &gl::Gl, shader_type: GLenum, source_file: &str) -> GLuint {

    let shader = gl.CreateShader(shader_type);

    let shader_code = match fs::read_to_string(source_file) {
        Ok(f) => f,
        Err(e) => panic!("error reading shader source file: {}: {}", source_file, e),
    };

    let source_c_str = CString::new(shader_code.as_bytes()).unwrap();
    gl.ShaderSource(shader, 1, &(source_c_str.as_ptr()), &(shader_code.len() as GLint));
    gl.CompileShader(shader);

    let mut status = gl::FALSE as GLint;
    gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) -1);

        gl.GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}: {}",source_file, std::str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
    }
    shader
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
