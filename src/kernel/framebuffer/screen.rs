use multiboot2::FramebufferTag;

use crate::framebuffer::{OutOfBoundsError, usize_plus_isize};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color(u8, u8, u8, u8);

impl Color {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, alpha: u8) -> Self {
        Self(b, g, r, alpha)
    }
}

#[derive(Debug)]
pub struct ScreenBuffer {
    max_x: usize,
    max_y: usize,
    pitch: usize,
    bpp: usize,
    buffer: &'static mut [Color],
}

// TODO : Implement a Buffer trait to account for different bpp (also more Color types)
// Also take the padding into account by checking pitch against width * bpp / 8
// Also put the Color type in private and add method to buffer to create a new pixel of Color to check that the color asked is supported (alpha channel, etc.)
// Also take into account the offset given by the framebuffer tag
impl ScreenBuffer {
    #[must_use]
    pub const fn new(framebuffer_tag: &FramebufferTag) -> Self {
        let width = framebuffer_tag.width() as usize;
        let height = framebuffer_tag.height() as usize;
        let pitch = framebuffer_tag.pitch() as usize;
        let bpp = framebuffer_tag.bpp() as usize;
        let len = (height * pitch) / (bpp / 8);
        Self {
            max_x: width,
            max_y: height,
            pitch,
            bpp,
            buffer: unsafe {
                core::slice::from_raw_parts_mut(framebuffer_tag.address() as *mut Color, len)
            },
        }
    }

    const fn coord_to_pxl_offset(&self, x: usize, y: usize) -> usize {
        y * self.pitch / (self.bpp / 8) + x
    }

    fn pxl(&mut self, x: usize, y: usize) -> Result<&mut Color, OutOfBoundsError> {
        self.buffer
            .get_mut(self.coord_to_pxl_offset(x, y))
            .ok_or_else(|| OutOfBoundsError::new_point(x, y, self.max_x, self.max_y))
    }

    /// # Errors
    /// This function will return an error if the point is out of bounds of the buffer.
    pub fn write(&mut self, x: usize, y: usize, color: Color) -> Result<(), OutOfBoundsError> {
        *self.pxl(x, y)? = color;
        Ok(())
    }

    /// # Errors
    /// This function will return an error if the point is out of bounds of the buffer.
    pub fn read(&mut self, x: usize, y: usize) -> Result<Color, OutOfBoundsError> {
        Ok(*self.pxl(x, y)?)
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn clear(&mut self, color: Color) {
        for y in 0..self.max_y {
            for x in 0..self.max_x {
                self.write(x, y, color).unwrap();
            }
        }
    }

    #[must_use]
    pub const fn max_y(&self) -> usize {
        self.max_y
    }

    #[must_use]
    pub const fn max_x(&self) -> usize {
        self.max_x
    }

    /// # Errors
    /// This function will return an error if the slice is out of bounds of the buffer.
    pub fn move_slice(
        &mut self,
        x: usize,
        y: usize,
        len: usize,
        dx: isize,
        dy: isize,
    ) -> Result<(), OutOfBoundsError> {
        if !(0..self.max_x).contains(&x)
            || !(0..self.max_y).contains(&y)
            || y * self.max_x + x + len > self.max_x * self.max_y
        {
            return Err(OutOfBoundsError::new_slice(
                y * self.max_x + x,
                y * self.max_x + x + len,
                self.max_x * self.max_y,
            ));
        } else if !(0..self.max_x).contains(&(usize_plus_isize(x, dx)))
            || !(0..self.max_y).contains(&(usize_plus_isize(y, dy)))
            || (usize_plus_isize(y, dy)) * self.max_x + usize_plus_isize(x, dx) + len
                > self.max_x * self.max_y
        {
            return Err(OutOfBoundsError::new_slice(
                (usize_plus_isize(y, dy)) * self.max_x + usize_plus_isize(x, dx),
                (usize_plus_isize(y, dy)) * self.max_x + usize_plus_isize(x, dx) + len,
                self.max_x * self.max_y,
            ));
        }

        let idx = self.coord_to_pxl_offset(x, y);
        let idx2 = self.coord_to_pxl_offset(usize_plus_isize(x, dx), usize_plus_isize(y, dy));
        self.buffer.copy_within(idx..idx + len, idx2);
        Ok(())
    }
}
