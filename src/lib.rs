extern crate heatmap;
extern crate hsl;
extern crate lodepng;
extern crate rusttype;

use std::path::Path;

use heatmap::Heatmap;
use hsl::HSL;
use lodepng::encode_file;
use rusttype::{FontCollection, PixelsXY, point, PositionedGlyph};

pub struct Waterfall {
    pub heatmap: Heatmap,
}

pub struct Config {
    num_slices: usize,
    precision: u32,
    slice_duration: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            precision: 2,
            num_slices: 300,
            slice_duration: 1,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Default::default()
    }

    pub fn num_slices(&mut self, count: usize) -> &mut Self {
        self.num_slices = count;
        self
    }

    pub fn slice_duration(&mut self, value: u64) -> &mut Self {
        self.slice_duration = value;
        self
    }

    pub fn build(self) -> Waterfall {
        Waterfall::configured(self)
    }
}

struct Label {
    value: u64,
    text: String,
}

impl Default for Waterfall {
    fn default() -> Waterfall {
        Waterfall::configured(Config::default())
    }
}

impl Waterfall {
    pub fn new() -> Waterfall {
        Default::default()
    }

    pub fn configure() -> Config {
        Config::default()
    }

    fn configured(config: Config) -> Waterfall {
        let heatmap = Heatmap::configure()
            .precision(config.precision)
            .num_slices(config.num_slices)
            .slice_duration(config.slice_duration)
            .build()
            .unwrap();
        Waterfall { heatmap: heatmap }
    }

    pub fn load_file(&mut self, file: String) {
        let mut d = Heatmap::load(file.clone());
        self.heatmap.merge(&mut d);
    }

    pub fn merge_heatmap(&mut self, mut heatmap: Heatmap) {
        self.heatmap.merge(&mut heatmap);
    }

    pub fn find_max(&mut self) -> u64 {
        let mut max = 0_u64;
        for slice in &self.heatmap {
            for bucket in &slice.histogram {
                if bucket.count() > max {
                    max = bucket.count();
                }
            }
        }
        max
    }

    pub fn render_png(&mut self, file: String) {
        let height = self.heatmap.num_slices() as usize;
        let width = self.heatmap.histogram_buckets() as usize;

        // build buffer from data
        let mut buffer = ImageBuffer::<ColorRgb>::new(width, height);
        let max = self.find_max();
        let mut x;
        let mut y = 0;

        // loop to color the pixels
        for slice in &self.heatmap {
            x = 0;
            for bucket in &slice.histogram {
                let pixel = color_from_value(bucket.count(), max);
                buffer.set_pixel(x, y, pixel);
                x += 1;
            }
        }

        // latency annotations
        let labels: Vec<Label> = vec!(
			Label{ value: 200, text: "200nS".to_string() },
			Label{ value: 500, text: "500nS".to_string() },
			Label{ value: 1000, text: "1uS".to_string() },
			Label{ value: 2000, text: "2uS".to_string() },
			Label{ value: 5000, text: "5uS".to_string() },
			Label{ value: 10000, text: "10uS".to_string() },
			Label{ value: 20000, text: "20uS".to_string() },
			Label{ value: 50000, text: "50uS".to_string() },
			Label{ value: 100000, text: "100uS".to_string() },
			Label{ value: 200000, text: "200uS".to_string() },
			Label{ value: 500000, text: "500uS".to_string() },
			Label{ value: 1000000, text: "1mS".to_string() },
			Label{ value: 2000000, text: "2mS".to_string() },
			Label{ value: 5000000, text: "5mS".to_string() },
			Label{ value: 10000000, text: "10mS".to_string() },
			Label{ value: 20000000, text: "20mS".to_string() },
			Label{ value: 50000000, text: "50mS".to_string() },
			Label{ value: 100000000, text: "100mS".to_string() },
			Label{ value: 200000000, text: "200mS".to_string() },
			Label{ value: 500000000, text: "500mS".to_string() },
			);

        let mut l = 0;
        y = 0;
        for slice in &self.heatmap {
            x = 0;
            for bucket in &slice.histogram {
                if (y % 60) == 0 {
                    if x == 0 {
                        let hour = y / 3600;
                        let minute = y / 60;
                        let time = format!("{:02}:{:02}", hour, minute);
                        let overlay = string_buffer(time.to_string(), 25.0);
                        buffer.overlay(&overlay, x, y);
                        buffer.horizontal_line(y, ColorRgb { r: 0, g: 0, b: 0 });
                    }
                    let v = bucket.value();
                    if l < labels.len() {
                        if v >= labels[l].value {
                            let overlay = string_buffer(labels[l].text.clone(), 25.0);
                            buffer.overlay(&overlay, x, y);
                            buffer.vertical_line(x, ColorRgb { r: 0, g: 0, b: 0 });
                            l += 1;
                        }
                    }

                }
                x += 1;
            }
        }

        let _ = buffer.write_png(file.clone());
    }
}

fn string_buffer(string: String, size: f32) -> ImageBuffer<ColorRgb> {
    // load font
    let font_data = include_bytes!("../assets/ubuntumono/UbuntuMono-Regular.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.into_font().unwrap();

    // size and scaling
    let height: f32 = size;
    let pixel_height = height.ceil() as usize;
    let scale = PixelsXY(height * 1.0, height);

    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let glyphs: Vec<PositionedGlyph> = font.layout(&string, scale, offset).collect();

    let width = glyphs.iter()
        .map(|g| g.h_metrics().advance_width)
        .fold(0.0, |x, y| x + y)
        .ceil() as usize;

    let mut overlay = ImageBuffer::<ColorRgb>::new(width, pixel_height);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = (x as i32 + bb.min.x) as usize;
                let y = (y as i32 + bb.min.y) as usize;
                if v > 0.25 {
                    overlay.set_pixel(x,
                                      y,
                                      ColorRgb {
                                          r: 255,
                                          g: 255,
                                          b: 255,
                                      });
                }
            })
        }
    }

    overlay
}

fn color_from_value(value: u64, max: u64) -> ColorRgb {
    let value = (value as f64) / (max as f64);

    let knee = 0.20_f64;

    let hsl: HSL;

    if value < knee {
        let l = 0.25_f64 + 0.25_f64 * value / knee;
        hsl = HSL {
            h: 236_f64,
            s: 1_f64,
            l: l,
        };
    } else {
        let h_per_deg: f64 = 236_f64 / (1.0_f64 - knee);
        let deg = (value - knee) * h_per_deg;

        hsl = HSL {
            h: (236_f64 - deg),
            s: 1_f64,
            l: 0.50_f64,
        };
    }

    let (r, g, b) = hsl.to_rgb();
    ColorRgb { r: r, g: g, b: b }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ColorRgb {
    r: u8,
    g: u8,
    b: u8,
}

struct ImageBuffer<T> {
    buffer: Vec<Vec<T>>,
    height: usize,
    width: usize,
}

impl ImageBuffer<ColorRgb> {
    pub fn new(width: usize, height: usize) -> ImageBuffer<ColorRgb> {
        let background = ColorRgb { r: 0, g: 0, b: 0 };
        let mut row = Vec::<ColorRgb>::with_capacity(width);
        for _ in 0..width {
            row.push(background);
        }
        let mut buffer = Vec::<Vec<ColorRgb>>::with_capacity(height);
        for _ in 0..height {
            buffer.push(row.clone());
        }
        ImageBuffer {
            buffer: buffer,
            height: height,
            width: width,
        }
    }

    pub fn write_png(self, file: String) -> Result<(), lodepng::ffi::Error> {
        let mut buffer = Vec::<u8>::with_capacity((self.height * self.width));
        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.buffer[row][col];
                buffer.push(pixel.r);
                buffer.push(pixel.g);
                buffer.push(pixel.b);
            }
        }
        let path = &Path::new(&file);
        encode_file(path, &buffer, self.width, self.height, lodepng::LCT_RGB, 8)
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: ColorRgb) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = value;
        }
    }

    pub fn overlay(&mut self, other: &ImageBuffer<ColorRgb>, x: usize, y: usize) {
        let ignore = ColorRgb { r: 0, g: 0, b: 0 };
        for sx in 0..other.width {
            for sy in 0..other.height {
                if other.buffer[sy][sx] != ignore {
                    if ((sy + y) < self.height) && ((sx + x) < self.width) {
                        self.buffer[(sy + y)][(sx + x)] = other.buffer[sy][sx];
                    }
                }
            }
        }
    }

    pub fn horizontal_line(&mut self, y: usize, color: ColorRgb) {
        for x in 0..self.width {
            self.buffer[y][x] = color;
        }
    }

    pub fn vertical_line(&mut self, x: usize, color: ColorRgb) {
        for y in 0..self.height {
            self.buffer[y][x] = color;
        }
    }
}
