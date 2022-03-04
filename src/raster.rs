pub struct Raster {
    /// Buffer of 0RGB values
    img_buf: Vec<u32>,
    z_buf: Vec<f32>,

    width: usize,
    height: usize,
}

impl Raster {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            img_buf: vec![0; width * height],
            z_buf: vec![1.0; width * height],
            width,
            height,
        }
    }

    /// Currently panics if the pixel is out of bounds
    pub fn set_pixel(&mut self, x: usize, y: usize, col: u32, z: f32) {
        let index = self.index(x, y);

        if index >= self.z_buf.len() {
            return;
        }

        if self.z_buf[index] > z {
            self.z_buf[index] = z;
            self.img_buf[index] = col;
        }
    }

    pub fn img_buf(&self) -> &[u32] {
        &self.img_buf
    }

    pub fn clear(&mut self) {
        self.img_buf.fill(0);
        self.z_buf.fill(1.0);
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}
