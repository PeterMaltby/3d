use anyhow::Result;
use gl::types::*;
use log::info;
use std::fmt;

use super::gl;

pub struct VertexArrayObjects {
    pub handle: u32,
}

impl VertexArrayObjects {
    pub fn new() -> Result<Self> {
        let vao = unsafe {
            let mut vao: GLuint = 0;
            gl::CreateVertexArrays(1, &mut vao);
            vao
        };
        info!("created vertex array objects");
        return Ok(VertexArrayObjects { handle: vao });
    }

    pub unsafe fn bind(&self) {
        info!("binding {}", self);
        gl::BindVertexArray(self.handle);
    }
}

impl Drop for VertexArrayObjects {
    fn drop(&mut self) {
        info!("deleting {}", self);
        unsafe {
            gl::DeleteVertexArrays(1, &self.handle);
        }
    }
}

impl fmt::Display for VertexArrayObjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex array object #{}", self.handle)
    }
}
