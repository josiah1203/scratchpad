use skia_safe::{
    AlphaType, Color, ColorType, Font, FontMgr, FontStyle, Image, ImageInfo, Paint, PaintStyle,
    Path, PathBuilder, RRect, Rect, SamplingOptions, surfaces,
};

const COLUMNS: usize = 32;
const MAX_TILE_COUNT: usize = 10_000;
const TILE_WIDTH: f32 = 168.0;
const TILE_HEIGHT: f32 = 126.0;
const TILE_GAP_X: f32 = 24.0;
const TILE_GAP_Y: f32 = 22.0;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { x: 28.0, y: 28.0, zoom: 0.82 }
    }
}

impl Camera {
    pub fn pan_by(&mut self, delta_x: f32, delta_y: f32) {
        self.x += delta_x;
        self.y += delta_y;
    }

    pub fn zoom_at(&mut self, focus: (f32, f32), wheel_delta: f32) {
        let factor = 1.12_f32.powf(wheel_delta);
        let world_x = (focus.0 - self.x) / self.zoom;
        let world_y = (focus.1 - self.y) / self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.12, 4.0);
        self.x = focus.0 - world_x * self.zoom;
        self.y = focus.1 - world_y * self.zoom;
    }
}

struct Tile {
    rect: Rect,
    fill: Color,
    label: String,
    curve: Option<Path>,
    has_image: bool,
}

pub struct Scene {
    tiles: Vec<Tile>,
    thumbnail: Image,
    font: Font,
}

impl Scene {
    /// Creates a synthetic scene capped at the spike's 10,000-tile workload.
    #[must_use]
    pub fn new(tile_count: usize) -> Self {
        Self {
            tiles: make_tiles(tile_count.min(MAX_TILE_COUNT)),
            thumbnail: make_thumbnail(),
            font: make_font(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn draw(&self, canvas: &skia_safe::Canvas, camera: Camera) {
        canvas.clear(Color::from_argb(255, 20, 24, 34));
        canvas.save();
        canvas.translate((camera.x, camera.y));
        canvas.scale((camera.zoom, camera.zoom));

        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        let mut border = Paint::default();
        border.set_anti_alias(true);
        border.set_style(PaintStyle::Stroke);
        border.set_stroke_width(1.25);
        let mut image_paint = Paint::default();
        image_paint.set_anti_alias(true);
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::from_argb(255, 219, 227, 241));
        for (index, tile) in self.tiles.iter().enumerate() {
            fill.set_color(tile.fill);
            canvas.draw_rrect(RRect::new_rect_xy(tile.rect, 10.0, 10.0), &fill);

            border.set_color(Color::from_argb(255, 79, 91, 112));
            canvas.draw_rrect(RRect::new_rect_xy(tile.rect, 10.0, 10.0), &border);

            if let Some(curve) = &tile.curve {
                fill.set_color(Color::from_argb(255, 246, 178, 107));
                fill.set_style(PaintStyle::Stroke);
                fill.set_stroke_width(3.0);
                canvas.draw_path(curve, &fill);
                fill.set_style(PaintStyle::Fill);
            }

            if tile.has_image {
                let image_rect =
                    Rect::from_xywh(tile.rect.left + 12.0, tile.rect.top + 12.0, 52.0, 52.0);
                canvas.draw_image_rect_with_sampling_options(
                    &self.thumbnail,
                    None,
                    image_rect,
                    SamplingOptions::default(),
                    &image_paint,
                );
            }

            let label_x =
                if tile.has_image { tile.rect.left + 74.0 } else { tile.rect.left + 12.0 };
            canvas.draw_str(&tile.label, (label_x, tile.rect.top + 36.0), &self.font, &text_paint);
            canvas.draw_str(
                if index % 2 == 0 { "VECTOR / TEXT" } else { "IMAGE / PATH" },
                (tile.rect.left + 12.0, tile.rect.bottom - 14.0),
                &self.font,
                &text_paint,
            );
        }

        canvas.restore();
    }
}

fn make_font() -> Font {
    let font_manager = FontMgr::default();
    let typeface = font_manager
        .match_family_style("Arial", FontStyle::default())
        .or_else(|| font_manager.match_family_style("sans-serif", FontStyle::default()))
        .or_else(|| font_manager.match_family_style("", FontStyle::default()))
        .expect("platform font manager should provide a default typeface");
    Font::new(typeface, 13.0)
}

fn make_tiles(tile_count: usize) -> Vec<Tile> {
    (0..tile_count)
        .map(|index| {
            let column = index % COLUMNS;
            let row = index / COLUMNS;
            let column = u16::try_from(column).expect("column is bounded by the grid width");
            let row = u16::try_from(row).expect("scene rows are bounded by MAX_TILE_COUNT");
            let x = f32::from(column) * (TILE_WIDTH + TILE_GAP_X);
            let y = f32::from(row) * (TILE_HEIGHT + TILE_GAP_Y);
            let shade = u8::try_from((index * 37) % 96).unwrap_or(0);
            let curve = if index % 5 == 0 {
                let mut path = PathBuilder::new();
                path.move_to((x + 14.0, y + 86.0));
                path.cubic_to((x + 42.0, y + 58.0), (x + 90.0, y + 112.0), (x + 148.0, y + 76.0));
                Some(path.detach())
            } else {
                None
            };

            Tile {
                rect: Rect::from_xywh(x, y, TILE_WIDTH, TILE_HEIGHT),
                fill: Color::from_argb(255, 38 + shade / 3, 48 + shade / 4, 67 + shade / 5),
                label: format!("OBJECT {index:05}"),
                curve,
                has_image: index % 9 == 0,
            }
        })
        .collect()
}

fn make_thumbnail() -> Image {
    let mut surface = surfaces::raster_n32_premul((64, 64)).expect("64x64 raster surface");
    let canvas = surface.canvas();
    canvas.clear(Color::from_argb(255, 30, 43, 71));

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    for row in 0_u8..8 {
        for column in 0_u8..8 {
            let light = (row + column) % 2 == 0;
            paint.set_color(if light {
                Color::from_argb(255, 79, 193, 182)
            } else {
                Color::from_argb(255, 72, 112, 179)
            });
            canvas.draw_rect(
                Rect::from_xywh(f32::from(column) * 8.0, f32::from(row) * 8.0, 8.0, 8.0),
                &paint,
            );
        }
    }

    surface.image_snapshot()
}

/// Rasterizes the scene to premultiplied RGBA pixels.
///
/// # Errors
///
/// Returns an error for zero or out-of-range dimensions, buffer-size overflow, or if Skia cannot
/// create the raster target.
pub fn render_rgba(
    scene: &Scene,
    camera: Camera,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("render dimensions must be non-zero".to_owned());
    }

    let width_i32 = i32::try_from(width).map_err(|_| "render width exceeds Skia limits")?;
    let height_i32 = i32::try_from(height).map_err(|_| "render height exceeds Skia limits")?;
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(4))
        .ok_or("render row is too large")?;
    let buffer_len = row_bytes
        .checked_mul(usize::try_from(height).map_err(|_| "render height is invalid")?)
        .ok_or("render buffer is too large")?;
    let mut pixels = vec![0_u8; buffer_len];
    let info =
        ImageInfo::new((width_i32, height_i32), ColorType::RGBA8888, AlphaType::Premul, None);
    let mut surface = surfaces::wrap_pixels(&info, &mut pixels, Some(row_bytes), None)
        .ok_or("Skia could not wrap the RGBA render target")?;
    scene.draw(surface.canvas(), camera);
    drop(surface);
    Ok(pixels)
}

/// Rasterizes the scene and encodes the result as PNG.
///
/// # Errors
///
/// Returns an error for zero or out-of-range dimensions, if Skia cannot allocate the raster
/// target, or if PNG encoding is unavailable.
pub fn render_png(
    scene: &Scene,
    camera: Camera,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("render dimensions must be non-zero".to_owned());
    }
    let width_i32 = i32::try_from(width).map_err(|_| "render width exceeds Skia limits")?;
    let height_i32 = i32::try_from(height).map_err(|_| "render height exceeds Skia limits")?;
    let mut surface = surfaces::raster_n32_premul((width_i32, height_i32))
        .ok_or("Skia could not allocate the benchmark surface")?;
    scene.draw(surface.canvas(), camera);
    surface
        .image_snapshot()
        .encode(None, skia_safe::EncodedImageFormat::PNG, Some(100))
        .map(|data| data.as_bytes().to_vec())
        .ok_or_else(|| "Skia PNG encoding is unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        AlphaType, Camera, Color, ColorType, ImageInfo, Paint, Rect, Scene, render_rgba, surfaces,
    };

    const GOLDEN_PPM: &str =
        include_str!("../../../tests/fixtures/golden-render/skia-vector-8x8.ppm");

    #[test]
    fn raster_scene_is_repeatable_and_contains_rendered_pixels() {
        let scene = Scene::new(128);
        let first = render_rgba(&scene, Camera::default(), 640, 480).unwrap();
        let second = render_rgba(&scene, Camera::default(), 640, 480).unwrap();

        assert_eq!(first, second);
        assert!(first.chunks_exact(4).any(|pixel| pixel != [20, 24, 34, 255]));

        let text_is_visible = (40..80).any(|y| {
            (80..160).any(|x| {
                let offset = (y * 640 + x) * 4;
                first[offset] > 180 && first[offset + 1] > 190 && first[offset + 2] > 210
            })
        });
        assert!(text_is_visible, "the first card label should be rasterized");
    }

    #[test]
    fn cpu_raster_matches_the_checked_in_pixel_golden() {
        const WIDTH: usize = 8;
        const HEIGHT: usize = 8;
        let row_bytes = WIDTH * 4;
        let mut pixels = vec![0_u8; row_bytes * HEIGHT];
        let info = ImageInfo::new(
            (i32::try_from(WIDTH).unwrap(), i32::try_from(HEIGHT).unwrap()),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let mut surface = surfaces::wrap_pixels(&info, &mut pixels, Some(row_bytes), None)
            .expect("Skia should wrap the golden pixel buffer");
        let canvas = surface.canvas();
        canvas.clear(Color::from_argb(255, 12, 16, 24));

        let mut paint = Paint::default();
        paint.set_anti_alias(false);
        paint.set_color(Color::from_argb(255, 220, 60, 80));
        canvas.draw_rect(Rect::from_xywh(1.0, 1.0, 4.0, 3.0), &paint);
        paint.set_color(Color::from_argb(255, 45, 160, 130));
        canvas.draw_rect(Rect::from_xywh(3.0, 3.0, 4.0, 4.0), &paint);
        drop(surface);

        let expected = parse_ascii_ppm(GOLDEN_PPM);
        assert_eq!((expected.0, expected.1), (WIDTH, HEIGHT));
        assert_eq!(expected.2.len(), WIDTH * HEIGHT);

        for (index, expected_rgb) in expected.2.iter().enumerate() {
            let offset = index * 4;
            let actual_rgb = [pixels[offset], pixels[offset + 1], pixels[offset + 2]];
            let x = index % WIDTH;
            let y = index / WIDTH;
            assert_eq!(actual_rgb, *expected_rgb, "CPU raster differs from golden at ({x}, {y})");
            assert_eq!(pixels[offset + 3], 255, "unexpected alpha at ({x}, {y})");
        }
    }

    fn parse_ascii_ppm(ppm: &str) -> (usize, usize, Vec<[u8; 3]>) {
        let tokens: Vec<_> = ppm.split_whitespace().collect();
        assert_eq!(tokens.first(), Some(&"P3"), "golden must be ASCII PPM");
        let width = tokens[1].parse().expect("PPM width should be an integer");
        let height = tokens[2].parse().expect("PPM height should be an integer");
        assert_eq!(tokens[3], "255", "golden should use 8-bit RGB samples");
        let samples: Vec<u8> = tokens[4..]
            .iter()
            .map(|sample| sample.parse().expect("PPM samples should be integers"))
            .collect();
        assert_eq!(samples.len(), width * height * 3, "invalid PPM sample count");
        let pixels = samples.chunks_exact(3).map(|rgb| [rgb[0], rgb[1], rgb[2]]).collect();
        (width, height, pixels)
    }

    #[test]
    fn render_dimensions_are_bounded_and_nonzero() {
        let scene = Scene::new(0);
        assert!(render_rgba(&scene, Camera::default(), 0, 10).is_err());
        assert!(render_rgba(&scene, Camera::default(), u32::MAX, u32::MAX).is_err());
    }

    #[test]
    fn synthetic_scene_is_capped_to_the_benchmark_limit() {
        assert_eq!(Scene::new(usize::MAX).len(), 10_000);
    }

    #[test]
    fn camera_pan_moves_the_view_by_the_pointer_delta() {
        let mut camera = Camera::default();
        camera.pan_by(18.0, -7.0);
        assert_eq!((camera.x, camera.y), (46.0, 21.0));
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor_fixed() {
        let mut camera = Camera::default();
        let focus = (437.0, 291.0);
        let world_before = ((focus.0 - camera.x) / camera.zoom, (focus.1 - camera.y) / camera.zoom);

        camera.zoom_at(focus, 2.0);

        let world_after = ((focus.0 - camera.x) / camera.zoom, (focus.1 - camera.y) / camera.zoom);
        assert!((world_before.0 - world_after.0).abs() < 0.0001);
        assert!((world_before.1 - world_after.1).abs() < 0.0001);
    }

    #[test]
    fn zoom_is_clamped_to_the_supported_range() {
        let mut camera = Camera::default();
        camera.zoom_at((0.0, 0.0), 100.0);
        assert!((camera.zoom - 4.0).abs() < f32::EPSILON);
        camera.zoom_at((0.0, 0.0), -100.0);
        assert!((camera.zoom - 0.12).abs() < f32::EPSILON);
    }
}
