use anyhow::Result;
use gl::types::*;
use log::info;
use std::fmt;
use std::marker::PhantomData;
use std::ffi::{c_void, CStr, CString};

use super::gl;

// TODO need to guarentee T is repr C
pub struct VertexBufferObjects<T> {
    pub handle: u32,
    memory_size: isize,
    buffer_type: PhantomData<T>,
}

impl<T> VertexBufferObjects<T> {
    pub fn new() -> Result<Self> {
        // TODO would be nice to guard T to Repr C but not sure i can?
        let memory_size = size_of::<T>() as isize;

        let vertex_buffer_object = unsafe {
            let mut vbo: GLuint = 0;
            gl::CreateBuffers(1, &mut vbo);

            gl::NamedBufferStorage(vbo, memory_size, std::ptr::null(), gl::DYNAMIC_STORAGE_BIT);
            vbo
        };

        info!("created vertex buffer object #{} with memory size {}", vertex_buffer_object, memory_size);

        return Ok(VertexBufferObjects::<T> {
            handle: vertex_buffer_object,
            memory_size,
            buffer_type: PhantomData::<T>,
        });
    }

    pub unsafe fn bind(&self) {
        info!("binding {}", self);
        gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, self.handle, 0, self.memory_size);
    }

    pub unsafe fn sub_buffer(&self, mut data: T) {
        let per_frame_data_ptr: *mut c_void = &mut data as *mut _ as *mut c_void;
        gl::NamedBufferSubData(self.handle, 0, self.memory_size, per_frame_data_ptr);
    }
}

impl<T> Drop for VertexBufferObjects<T> {
    fn drop(&mut self) {
        info!("deleting {}", self);
        unsafe {
            gl::DeleteVertexArrays(1, &self.handle);
        }
    }
}

impl<T> fmt::Display for VertexBufferObjects<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex buffer object #{}", self.handle)
    }
}
