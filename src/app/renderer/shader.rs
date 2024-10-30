use anyhow::{anyhow, Context, Error, Result};
use gl::types::*;
use log::{debug, info};
use std::ffi::{c_void, CStr, CString};
use std::fs;

use super::gl;

pub struct Shader{ 
    pub handle: u32,
}

pub enum ShaderType {
    FRAGMENT = gl::FRAGMENT_SHADER as isize,
    VERTEX = gl::VERTEX_SHADER as isize,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source_file: &str) -> Result<Self> {

        info!("compiling shader: {}", source_file);
        let handle = unsafe {
            let shader = gl::CreateShader(shader_type as u32);

            let shader_code = fs::read_to_string(source_file).context("failed to read texture source file")?;

            let source_c_str = CString::new(shader_code.as_bytes()).context("failed to convert shader source to c string")?;

            gl::ShaderSource(shader, 1, &(source_c_str.as_ptr()), &(shader_code.len() as GLint));
            gl::CompileShader(shader);

            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
                let mut log_buf: Vec<u8> = Vec::with_capacity(log_len as usize);
                log_buf.set_len((log_len as usize) - 1);
                gl::GetShaderInfoLog(shader, log_len, std::ptr::null_mut(), log_buf.as_mut_ptr() as *mut GLchar);

                return Err(anyhow!(String::from_utf8(log_buf).unwrap().to_string()));
            }
            shader
        };

        info!("created shader {}: {}", handle, source_file);

        Ok(Shader { handle })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        debug!("deleting shader: {}", self.handle);
        unsafe { gl::DeleteShader(self.handle);}
    }
}
