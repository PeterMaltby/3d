use anyhow::{anyhow, Context, Result};
use gl::types::*;
use log::info;
use shader::Shader;
use std::fmt;

use super::gl;
use super::shader;

pub struct Program {
    pub handle: u32,
}

impl Program {
    pub fn new(vertex_shader: &Shader, fragment_shader: &Shader) -> Result<Self> {

        let program_id = unsafe {
            let program_id = gl::CreateProgram();
            if program_id == 0 {
                return Err(anyhow!("glCreateProgram failed: {}, {}", vertex_shader, fragment_shader));
            }

            // check Shaders are valid code

            gl::AttachShader(program_id, vertex_shader.handle);
            gl::AttachShader(program_id, fragment_shader.handle);

            gl::LinkProgram(program_id);

            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log_buf: Vec<u8> = Vec::with_capacity(log_len as usize);
                log_buf.set_len((log_len as usize) - 1);
                gl::GetProgramInfoLog(program_id, log_len, std::ptr::null_mut(), log_buf.as_mut_ptr() as *mut GLchar);

                return Err(anyhow!(String::from_utf8(log_buf).unwrap().to_string()))
                    .context(format!("glLinkPorgram failed {}, {}", vertex_shader, fragment_shader));
            }
            program_id
        };

        info!("created program #{} from {}, {}",program_id, vertex_shader, fragment_shader);

        return Ok(Program { handle: program_id });
    }

    pub unsafe fn use_program(&self) {
        info!("using program: {}", self);
        gl::UseProgram(self.handle);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        info!("deleting {}", self);
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "program #{}", self.handle)
    }
}
