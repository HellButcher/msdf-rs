use image::{GrayImage, Luma};
use msdf::{raster::Rasterizer, shape::Shape};
use ttf_parser::{Face, GlyphId};

const SIZE: f64 = 100.0;
const OFFSET: u8 = 16;

fn glyph_shape(face: &Face, glyph_id: GlyphId) -> Shape<f64> {
    let mut builder = Shape::builder();
    face.outline_glyph(glyph_id, &mut builder);
    builder.finish().unwrap()
}

fn raster_bitmap(face: &Face, c: char) -> GrayImage {
    let scale = SIZE / face.units_per_em().unwrap_or(1024) as f64;
    let glyph_id = face.glyph_index(c).unwrap();
    let shape = glyph_shape(&face, glyph_id);

    let width = ((shape.aabr().max.x - shape.aabr().min.x) * scale) as usize + 1;
    let height = ((shape.aabr().max.y - shape.aabr().min.y) * scale) as usize + 1;
    let mut image = GrayImage::new(width as u32, height as u32);
    let put_pixel = |x: usize, y: usize, filled: bool| {
        let color;
        if filled {
            color = Luma([0]);
        } else {
            color = Luma([255]);
        }
        image.put_pixel(x as u32, y as u32, color);
    };

    Rasterizer::new()
        .with_scale(scale)
        .with_translate(-shape.aabr().min.x, -shape.aabr().min.y)
        .rasterize_bitmap(&shape, width, height, put_pixel);

    image
}

fn raster_sdf(face: &Face, c: char) -> GrayImage {
    let scale = SIZE / face.units_per_em().unwrap_or(1024) as f64;
    let glyph_id = face.glyph_index(c).unwrap();
    let shape = glyph_shape(&face, glyph_id);

    let width = ((shape.aabr().max.x - shape.aabr().min.x) * scale) as usize + OFFSET as usize * 2;
    let height = ((shape.aabr().max.y - shape.aabr().min.y) * scale) as usize + OFFSET as usize * 2;
    let mut image = GrayImage::new(width as u32, height as u32);
    let put_pixel = |x: usize, y: usize, value: u8| {
        image.put_pixel(x as u32, y as u32, Luma([value]));
    };

    Rasterizer::new()
        .with_scale(scale)
        .with_translate(
            -shape.aabr().min.x + OFFSET as f64 / scale,
            -shape.aabr().min.y + OFFSET as f64 / scale,
        )
        .rasterize_sdf(&shape, width, height, OFFSET, put_pixel);

    image
}

fn main() {
    let font = include_bytes!("fonts/OpenSans-Regular.ttf");
    let face = Face::from_slice(font, 0).unwrap();
    let examples_dest = std::path::Path::new("examples/out");
    std::fs::create_dir_all(examples_dest).unwrap();
    println!("Writing example outputs to {}", examples_dest.display());

    raster_bitmap(&face, '#').save(examples_dest.join("bitmap_a.png")).unwrap();
    raster_bitmap(&face, '@').save(examples_dest.join("bitmap_b.png")).unwrap();
    raster_bitmap(&face, 'R').save(examples_dest.join("bitmap_c.png")).unwrap();

    raster_sdf(&face, '#').save(examples_dest.join("sdf_a.png")).unwrap();
    raster_sdf(&face, '@').save(examples_dest.join("sdf_b.png")).unwrap();
    raster_sdf(&face, 'R').save(examples_dest.join("sdf_c.png")).unwrap();
}
