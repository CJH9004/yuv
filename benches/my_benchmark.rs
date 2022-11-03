use conv::ValueInto;
use criterion::{criterion_group, criterion_main, Criterion};

use std::{fs::File, io::Read};

use image::{GenericImage, ImageBuffer, Pixel, Rgb, RgbImage};
use imageproc::{
    definitions::Clamp,
    drawing::{draw_hollow_rect_mut, draw_text_mut},
    rect::Rect,
};
use rusttype::{Font, Scale};

use yuv::*;

fn draw_box<T: GenericImage>(
    img: &mut T,
    font: &Font,
    rect: Rect,
    text: &str,
    scale: Scale,
    color: T::Pixel,
) where
    <T::Pixel as Pixel>::Subpixel: ValueInto<f32> + Clamp<f32>,
{
    draw_hollow_rect_mut(img, rect, color);
    draw_text_mut(img, color, rect.top(), rect.left(), scale, &font, text);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut yuv_file = File::open("data/1.yuv").unwrap();
    let mut yuv_buf = Vec::new();
    yuv_file.read_to_end(&mut yuv_buf).unwrap();
    let mut nv12 = NV12Image::from(yuv_buf, 1920, 1080);

    let mut rgb: RgbImage = ImageBuffer::new(1920, 1080);

    let font_data: &[u8] = include_bytes!("../data/fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
    let font = Font::try_from_bytes(font_data).unwrap();

    let rect = Rect::at(101, 100).of_size(201, 100);
    let text = "测试";
    let scale = Scale::uniform(48.);

    c.bench_function("draw_box_on_rgb", |b| {
        b.iter(|| draw_box(&mut rgb, &font, rect, text, scale, Rgb([0, 0, 0])))
    });

    c.bench_function("draw_box_on_nv12", |b| {
        b.iter(|| draw_box(&mut nv12, &font, rect, text, scale, BLACK))
    });

    let mut nv12 = NV12Image2(nv12);
    let rect2 = Rect::at(101 / 2, 100 / 2).of_size(201 / 2, 100 / 2);
    let scale2 = Scale::uniform(48. / 2.0);
    c.bench_function("draw_box_on_nv12_2", |b| {
        b.iter(|| draw_box(&mut nv12, &font, rect2, text, scale2, BLACK))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
