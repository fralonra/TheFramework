use crate::prelude::*;

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct TheRGBABuffer {
    dim: TheDim,

    buffer: Vec<u8>,
}

/// TheRGBABuffer contains the pixel buffer for a canvas or icon.
impl TheRGBABuffer {
    /// Create an empty buffer.
    pub fn empty() -> Self {
        Self {
            dim: TheDim::zero(),
            buffer: vec![],
        }
    }

    /// Creates a buffer of the given dimension.
    pub fn new(dim: TheDim) -> Self {
        Self {
            dim,
            buffer: vec![0; dim.width as usize * dim.height as usize * 4],
        }
    }

    /// Creates a buffer from existing data.
    pub fn from(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            dim: TheDim::new(0, 0, width as i32, height as i32),
            buffer,
        }
    }

    /// Check for size validity
    pub fn is_valid(&self) -> bool {
        self.dim.is_valid()
    }

    /// Gets the width (stride) of the buffer.
    pub fn dim(&self) -> &TheDim {
        &self.dim
    }

    /// Gets the width (stride) of the buffer.
    pub fn stride(&self) -> usize {
        self.dim.width as usize
    }

    /// Gets a slice of the buffer.
    pub fn pixels(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Gets a mutable slice of the buffer.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }

    /// Set the dimension of the buffer.
    pub fn set_dim(&mut self, dim: TheDim) {
        if dim != self.dim {
            self.dim = dim;
            self.allocate();
        }
    }

    /// Allocates the buffer.
    pub fn allocate(&mut self) {
        if self.dim.is_valid() {
            self.buffer = vec![0; self.dim.width as usize * self.dim.height as usize * 4];
        } else {
            self.buffer = vec![];
        }
    }

    /// Copy the other buffer into this buffer at the given coordinates.
    pub fn copy_into(&mut self, x: i32, y: i32, other: &TheRGBABuffer) {
        let dest = &mut self.buffer[..];
        let width = (other.dim.width * 4) as usize;
        for h in 0..other.dim.height {
            let s = (h * other.dim.width * 4) as usize;
            let d = ((h + y) * self.dim.width * 4 + x * 4) as usize;
            dest[d..d + width].copy_from_slice(&other.buffer[s..s + width]);
        }
    }
}