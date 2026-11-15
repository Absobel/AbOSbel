use alloc::vec::Vec;
use multiboot2::FramebufferTag;
use spin::MutexGuard;

use crate::{
    drivers::framebuffer::{BUFFER, OutOfBoundsError},
    memory::VirtualAddress,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color(u8, u8, u8, u8);

impl Color {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, alpha: u8) -> Self {
        Self(b, g, r, alpha)
    }
}

pub struct Canvas<'a>(MutexGuard<'a, ScreenBuffer>);

impl Canvas<'_> {
    pub fn max_x(&self) -> usize {
        self.0.max_x()
    }

    pub fn max_y(&self) -> usize {
        self.0.max_y()
    }

    pub fn write(&mut self, x: usize, y: usize, color: Color) -> Result<(), OutOfBoundsError> {
        self.0.write(x, y, color)
    }

    #[expect(dead_code)]
    pub fn fill_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: Color,
    ) -> Result<(), OutOfBoundsError> {
        self.0.fill_rect(x, y, w, h, color)
    }

    #[expect(dead_code)]
    pub fn read(&mut self, x: usize, y: usize) -> Result<Color, OutOfBoundsError> {
        self.0.read(x, y)
    }

    pub fn move_slice(
        &mut self,
        x: usize,
        y: usize,
        len: usize,
        dx: isize,
        dy: isize,
    ) -> Result<(), OutOfBoundsError> {
        self.0.move_slice(x, y, len, dx, dy)
    }

    pub fn clear(&mut self, color: Color) {
        self.0.clear(color);
    }
}

impl Drop for Canvas<'_> {
    fn drop(&mut self) {
        self.0.flush();
    }
}

/// Reminder: the lowest functions that modify the buffer are `write` and `move_slice`
#[derive(Debug)]
pub struct ScreenBuffer {
    max_x: usize,
    max_y: usize,
    pitch: usize,
    bpp: usize,
    backbuffer: Vec<Color>,
    screenbuffer: &'static mut [Color],
    dirty: ((usize, usize), (usize, usize)), // (left, right) and (top, bottom) of the dirty area
}

// TODO : Implement a Buffer trait to account for different bpp (also more Color types)
// Also take the padding into account by checking pitch against width * bpp / 8
// Also put the Color type in private and add method to buffer to create a new pixel of Color to check that the color asked is supported (alpha channel, etc.)
// Also take into account the offset given by the framebuffer tag
impl ScreenBuffer {
    const DIRTY_INIT: ((usize, usize), (usize, usize)) = ((usize::MAX, 0), (usize::MAX, 0));

    #[must_use]
    pub fn new(framebuffer_tag: &FramebufferTag, addr: VirtualAddress) -> Self {
        let width = framebuffer_tag.width() as usize;
        let height = framebuffer_tag.height() as usize;
        let pitch = framebuffer_tag.pitch() as usize;
        let bpp = framebuffer_tag.bpp() as usize;
        let len = (height * pitch) / (bpp / 8);
        let address = addr;
        Self {
            max_x: width,
            max_y: height,
            pitch,
            bpp,
            dirty: Self::DIRTY_INIT,
            backbuffer: alloc::vec![Color(0, 0, 0, 0); len],
            screenbuffer: unsafe { core::slice::from_raw_parts_mut(address.as_mut_ptr(), len) },
        }
    }

    pub fn canvas() -> Canvas<'static> {
        Canvas(
            BUFFER
                .get()
                .expect("framebuffer has to be initialized")
                .lock(),
        )
    }

    const fn coord_to_pxl_offset(&self, x: usize, y: usize) -> usize {
        y * self.pitch / (self.bpp / 8) + x
    }

    fn pxl(&mut self, x: usize, y: usize) -> Result<&mut Color, OutOfBoundsError> {
        let index = self.coord_to_pxl_offset(x, y);
        self.backbuffer
            .get_mut(index)
            .ok_or_else(|| OutOfBoundsError::new_point(x, y, self.max_x, self.max_y))
    }

    fn update_dirty(&mut self, x: usize, y: usize, w: usize, h: usize) {
        assert!(x < self.max_x && x + w <= self.max_x && y < self.max_y && y + h <= self.max_y);
        let (left, right) = self.dirty.0;
        let (top, bottom) = self.dirty.1;
        self.dirty.0 = (left.min(x), right.max(x + w));
        self.dirty.1 = (top.min(y), bottom.max(y + h));
    }

    /// # Errors
    /// This function will return an error if the point is out of bounds of the buffer.
    fn write(&mut self, x: usize, y: usize, color: Color) -> Result<(), OutOfBoundsError> {
        *self.pxl(x, y)? = color;
        self.update_dirty(x, y, 1, 1);
        Ok(())
    }

    fn fill_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: Color,
    ) -> Result<(), OutOfBoundsError> {
        for y in y..y + h {
            for x in x..x + w {
                self.write(x, y, color)?;
            }
        }
        Ok(())
    }

    /// # Errors
    /// This function will return an error if the point is out of bounds of the buffer.
    fn read(&mut self, x: usize, y: usize) -> Result<Color, OutOfBoundsError> {
        Ok(*self.pxl(x, y)?)
    }

    fn clear(&mut self, color: Color) {
        for y in 0..self.max_y {
            for x in 0..self.max_x {
                self.write(x, y, color).unwrap();
            }
        }
    }

    #[must_use]
    const fn max_y(&self) -> usize {
        self.max_y
    }

    #[must_use]
    const fn max_x(&self) -> usize {
        self.max_x
    }

    /// # Errors
    /// This function will return an error if the slice is out of bounds of the buffer.
    fn move_slice(
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
        } else if !(0..self.max_x).contains(&(x.wrapping_add_signed(dx)))
            || !(0..self.max_y).contains(&(y.wrapping_add_signed(dy)))
            || (y.wrapping_add_signed(dy)) * self.max_x + (x.wrapping_add_signed(dx)) + len
                > self.max_x * self.max_y
        {
            return Err(OutOfBoundsError::new_slice(
                (y.wrapping_add_signed(dy)) * self.max_x + (x.wrapping_add_signed(dx)),
                (y.wrapping_add_signed(dy)) * self.max_x + (x.wrapping_add_signed(dx)) + len,
                self.max_x * self.max_y,
            ));
        }

        let idx = self.coord_to_pxl_offset(x, y);
        let idx2 = self.coord_to_pxl_offset(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
        self.backbuffer.copy_within(idx..idx + len, idx2);

        let dest_x = x.wrapping_add_signed(dx);
        let dest_y = y.wrapping_add_signed(dy);
        self.update_dirty(0, dest_y, self.max_x, (dest_x + len).div_ceil(self.max_x));

        Ok(())
    }

    // TODO: one day do it on a timer, maybe
    fn flush(&mut self) {
        if self.dirty.0.0 >= self.dirty.0.1 || self.dirty.1.0 >= self.dirty.1.1 {
            return;
        }
        let (left, right) = self.dirty.0;
        let (top, bottom) = self.dirty.1;
        for y in top..bottom {
            let source_start = self.coord_to_pxl_offset(left, y);
            let source_end = self.coord_to_pxl_offset(right, y);
            let dest_start = self.coord_to_pxl_offset(left, y);
            let dest_end = self.coord_to_pxl_offset(right, y);
            self.screenbuffer[dest_start..dest_end]
                .copy_from_slice(&self.backbuffer[source_start..source_end]);
        }
        self.dirty = Self::DIRTY_INIT;
    }
}
