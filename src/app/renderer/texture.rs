use anyhow::{anyhow, Result};
use gl::types::*;
use image::ImageReader;
use log::info;
use std::ffi::c_void;
use std::fmt;

use super::gl;

pub struct Texture {
    pub handle: u32,
}

impl Texture {
    pub fn new(source_file: &str) -> Result<Self> {
        let texture_id = unsafe {
            let img = match ImageReader::open(source_file) {
                Ok(img) => img,
                Err(e) => return Err(anyhow!("failed to read source file {}", source_file).context(e)),
            };
            let img = match img.decode() {
                Ok(img) => img,
                Err(e) => return Err(anyhow!("failed to decode image source file {}", source_file).context(e)),
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
            gl::TextureSubImage2D(
                tex,
                0,
                0,
                0,
                width,
                height,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                (&img as &[u8]).as_ptr() as *const c_void,
            );

            tex
        };
        info!("creating texture #{} from {}", texture_id, source_file);

        return Ok(Texture { handle: texture_id });
    }

    pub unsafe fn bind(&self) {
        info!("binding {}", self);
        gl::BindTextures(0, 1, &self.handle);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        info!("deleting {}", self);
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

impl fmt::Display for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "texture #{}", self.handle)
    }
}
