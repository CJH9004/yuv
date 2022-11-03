use std::ops::IndexMut;

use image::{GenericImage, GenericImageView, Luma, LumaA, Pixel, Rgb, Rgba};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct YUV(pub [u8; 3]);

pub const BLACK: YUV = YUV([0, 0x80, 0x80]);
pub const WHITE: YUV = YUV([0xff, 0x80, 0x80]);
pub const RED: YUV = YUV([0x4c, 0x55, 0xff]);
pub const GREEN: YUV = YUV([0, 0, 0]);
pub const CYAN: YUV = YUV([0xb3, 0xab, 0x00]);
pub const BLUE: YUV = YUV([0x1d, 0xff, 0x6b]);
pub const YELLOW: YUV = YUV([0xe2, 0x00, 0x95]);

impl YUV {
    fn rgb(&self) -> [u8; 3] {
        let y = self.0[0] as f32;
        let u = self.0[1] as f32;
        let v = self.0[2] as f32;
        let r = y + (140. * (v - 128.)) / 100.;
        let g = y - (34. * (u - 128.)) / 100. - (71. * (v - 128.)) / 100.;
        let b = y + (177. * (u - 128.)) / 100.;
        [r as u8, g as u8, b as u8]
    }
}

const DEFAULT_MAX_VALUE: u8 = 255;

impl Pixel for YUV {
    type Subpixel = u8;

    const CHANNEL_COUNT: u8 = 3;

    fn channels(&self) -> &[Self::Subpixel] {
        &self.0
    }

    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        &mut self.0
    }

    const COLOR_MODEL: &'static str = "YUV";

    fn channels4(
        &self,
    ) -> (
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
    ) {
        let mut channels = [DEFAULT_MAX_VALUE; 4];
        channels[0..Self::CHANNEL_COUNT as usize].copy_from_slice(&self.0);
        (channels[0], channels[1], channels[2], channels[3])
    }

    fn from_channels(
        a: Self::Subpixel,
        b: Self::Subpixel,
        c: Self::Subpixel,
        d: Self::Subpixel,
    ) -> Self {
        *<Self as Pixel>::from_slice(&[a, b, c, d][..Self::CHANNEL_COUNT as usize])
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        assert_eq!(slice.len(), Self::CHANNEL_COUNT as usize);
        unsafe { &*(slice.as_ptr() as *const Self) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self {
        assert_eq!(slice.len(), Self::CHANNEL_COUNT as usize);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Self) }
    }

    fn to_rgb(&self) -> Rgb<Self::Subpixel> {
        Rgb(self.rgb())
    }

    fn to_rgba(&self) -> Rgba<Self::Subpixel> {
        let mut channels = [DEFAULT_MAX_VALUE; 4];
        channels[0..Self::CHANNEL_COUNT as usize].copy_from_slice(&self.rgb());
        Rgba(channels)
    }

    fn to_luma(&self) -> Luma<Self::Subpixel> {
        Luma([self.rgb()[0]])
    }

    fn to_luma_alpha(&self) -> LumaA<Self::Subpixel> {
        LumaA([self.rgb()[0], DEFAULT_MAX_VALUE])
    }

    fn map<F>(&self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = (*self).clone();
        this.apply(f);
        this
    }

    fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        for v in &mut self.0 {
            *v = f(*v)
        }
    }

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = (*self).clone();
        this.apply_with_alpha(f, g);
        this
    }

    fn apply_with_alpha<F, G>(&mut self, f: F, _: G)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.apply(f)
    }

    fn map2<F>(&self, other: &Self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &Self, mut f: F)
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        for (a, &b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {
        let yuv = self.0;

        let max = DEFAULT_MAX_VALUE;

        let y = max - yuv[0];
        let u = max - yuv[1];
        let v = max - yuv[2];

        *self = Self([y, u, v])
    }

    fn blend(&mut self, other: &Self) {
        *self = *other
    }
}

pub struct NV12Image<T: IndexMut<usize, Output = u8>> {
    data: T,
    width: u32,
    height: u32,
    gray_size: u32,
}

impl<T: IndexMut<usize, Output = u8>> NV12Image<T> {
    fn check_bounds(&self, x: u32, y: u32) {
        if x >= self.width || y >= self.height {
            panic!(
                "Image index {:?} out of bounds {:?}",
                (x, y),
                (self.width, self.height)
            )
        }
    }

    fn to_zero_or_even(n: u32) -> u32 {
        n - n % 2
    }

    fn pixel_indices(&self, x: u32, y: u32) -> (usize, usize, usize) {
        let offset = y * self.width;
        let y_index = offset + x;
        let uv_index = self.gray_size + offset / 2 + x;
        (y_index as usize, uv_index as usize, uv_index as usize + 1)
    }

    pub fn from(data: T, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            gray_size: width * height,
        }
    }

    pub fn take_data(self) -> T {
        self.data
    }

    pub fn ref_data(&self) -> &T {
        &self.data
    }
}

impl<T: IndexMut<usize, Output = u8>> GenericImageView for NV12Image<T> {
    type Pixel = YUV;

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width, self.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.check_bounds(x, y);
        let x = Self::to_zero_or_even(x);
        let y = Self::to_zero_or_even(y);
        let indices = self.pixel_indices(x, y);
        YUV([
            self.data[indices.0],
            self.data[indices.1],
            self.data[indices.2],
        ])
    }
}

impl<T: IndexMut<usize, Output = u8>> GenericImage for NV12Image<T> {
    fn get_pixel_mut(&mut self, _: u32, _: u32) -> &mut Self::Pixel {
        todo!()
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.check_bounds(x, y);
        let x = Self::to_zero_or_even(x);
        let y = Self::to_zero_or_even(y);
        let indices = self.pixel_indices(x, y);
        self.data[indices.0] = pixel.0[0];
        self.data[indices.0 + 1] = pixel.0[0];
        self.data[indices.0 + self.width as usize] = pixel.0[0];
        self.data[indices.0 + self.width as usize + 1] = pixel.0[0];
        self.data[indices.1] = pixel.0[1];
        self.data[indices.2] = pixel.0[2];
    }

    fn blend_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.put_pixel(x, y, pixel)
    }
}

pub struct NV12Image2<T: IndexMut<usize, Output = u8>>(pub NV12Image<T>);

impl<T: IndexMut<usize, Output = u8>> GenericImageView for NV12Image2<T> {
    type Pixel = YUV;

    fn dimensions(&self) -> (u32, u32) {
        (self.0.width / 2, self.0.height / 2)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.0.width / 2, self.0.height / 2)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.0.get_pixel(x * 2, y * 2)
    }
}

impl<T: IndexMut<usize, Output = u8>> GenericImage for NV12Image2<T> {
    fn get_pixel_mut(&mut self, _: u32, _: u32) -> &mut Self::Pixel {
        todo!()
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.0.put_pixel(x * 2, y * 2, pixel)
    }

    fn blend_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.put_pixel(x, y, pixel)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use imageproc::{
        drawing::{draw_hollow_rect_mut, draw_text_mut},
        rect::Rect,
    };
    use rusttype::{Font, Scale};

    use super::*;
    #[test]
    fn draw_box() {
        let mut yuv_file = File::open("data/1.yuv").unwrap();
        let mut yuv_buf = Vec::new();
        yuv_file.read_to_end(&mut yuv_buf).unwrap();

        let mut img = NV12Image::from(yuv_buf, 1920, 1080);
        draw_hollow_rect_mut(&mut img, Rect::at(101, 100).of_size(201, 100), GREEN);
        let font_data: &[u8] = include_bytes!("../data/fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
        let font = Font::try_from_bytes(font_data).unwrap();
        draw_text_mut(&mut img, BLUE, 101, 101, Scale::uniform(48.), &font, "测试");

        let mut out_file = File::create("1.out.yuv").unwrap();
        out_file.write_all(img.ref_data()).unwrap();
        // ffmpeg -s 1920*1080 -pix_fmt nv12 -i 1.out.yuv 1.jpg -y
    }
    #[test]
    fn draw_box2() {
        let mut yuv_file = File::open("data/1.yuv").unwrap();
        let mut yuv_buf = Vec::new();
        yuv_file.read_to_end(&mut yuv_buf).unwrap();

        let mut img = NV12Image2(NV12Image::from(yuv_buf, 1920, 1080));
        draw_hollow_rect_mut(
            &mut img,
            Rect::at(101 / 2, 100 / 2).of_size(201 / 2, 100 / 2),
            GREEN,
        );
        let font_data: &[u8] = include_bytes!("../data/fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
        let font = Font::try_from_bytes(font_data).unwrap();
        draw_text_mut(
            &mut img,
            BLUE,
            101 / 2,
            101 / 2,
            Scale::uniform(48. / 2.),
            &font,
            "测试",
        );

        let mut out_file = File::create("1.out.yuv").unwrap();
        out_file.write_all(img.0.ref_data()).unwrap();
        // ffmpeg -s 1920*1080 -pix_fmt nv12 -i 1.out.yuv 1.jpg -y
    }
}
