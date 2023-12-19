use std::collections::HashMap;

use clap::Parser;
use image::{io::Reader as ImageReader, ImageError};
use palette::{cast::FromComponents, encoding::Srgb as TSrgb, rgb::Rgb, Hsv, IntoColor, Srgb};

type H = usize;
type S = usize;
type V = usize;

fn main() -> Result<(), ImageError> {
    let args = Cli::parse();

    let image = ImageReader::open(args.input)?.decode()?;
    let mut image = image.into_rgb8();

    let mut colors: HashMap<(H, S, V), ([u32; 3], usize)> = HashMap::new();
    let mut hues: HashMap<usize, usize> = HashMap::new();
    let mut hue_thresholds = args.hue_thresholds;
    hue_thresholds.sort_by(|a, b| a.partial_cmp(&b).expect(""));
    let saturation_chunk_range = 100. / args.saturation_chunks as f32;

    let mut use_luminance = false;
    // ? Luminance/value chunk range
    #[allow(unused_assignments)]
    let mut lv_chunk_range = 0.;

    if let Some(value_chunks) = args.value_chunks {
        lv_chunk_range = 100. / value_chunks as f32;
    } else if let Some(lumniance_chunks) = args.luminance_chunks {
        use_luminance = true;
        lv_chunk_range = 100. / lumniance_chunks as f32;
    } else {
        println!("Either luminance bounds or value bounds must be specified!");
        return Ok(());
    }

    // ? Compute RGB means
    for pixel in <&mut [Srgb<u8>]>::from_components(&mut *image) {
        let hsl_pixel: Hsv = pixel.into_format().into_color();

        let mut hue: f32 = hsl_pixel.hue.into();
        hue += 180.;
        let saturation = hsl_pixel.saturation * 100.;
        let luminance_or_value = if use_luminance {
            luminance(pixel) as f32
        } else {
            hsl_pixel.value
        } * 100.;

        let h = get_hue_index(hue as f64, &hue_thresholds);
        let s = (saturation / saturation_chunk_range) as usize;
        let lv = (luminance_or_value / lv_chunk_range) as usize;

        {
            let hue = hue as usize;
            if !hues.contains_key(&hue) {
                hues.insert(hue, 1_usize);
            } else {
                hues.insert(hue, hues[&hue] + 1);
            }
        }

        let index = (h, s, lv);

        if !colors.contains_key(&index) {
            colors.insert(index, ([0; 3], 0));
        }

        let (mut sum, mut count) = colors[&index];

        count += 1;
        sum[0] += pixel.red as u32;
        sum[1] += pixel.green as u32;
        sum[2] += pixel.blue as u32;

        colors.insert(index, (sum, count));
    }

    {
        let mut hues: Vec<(usize, usize)> = hues.into_iter().collect();
        hues.sort_by_key(|hue| hue.0);
    }

    // ? Cel shade
    for pixel in <&mut [Srgb<u8>]>::from_components(&mut *image) {
        let hsl_pixel: Hsv = pixel.into_format().into_color();

        let mut hue: f32 = hsl_pixel.hue.into();
        hue += 180.;
        let saturation = hsl_pixel.saturation * 100.;
        let luminance_or_value = if use_luminance {
            luminance(pixel) as f32
        } else {
            hsl_pixel.value
        } * 100.;

        let h = get_hue_index(hue as f64, &hue_thresholds);
        let s = (saturation / saturation_chunk_range) as usize;
        let lv = (luminance_or_value / lv_chunk_range) as usize;

        let index = (h, s, lv);

        let (sum, count) = colors[&index];
        let mean = [
            (sum[0] as f64 / count as f64) as u8,
            (sum[1] as f64 / count as f64) as u8,
            (sum[2] as f64 / count as f64) as u8,
        ];

        *pixel = mean.into();
    }

    image.save(args.output)?;

    Ok(())
}

fn get_hue_index(hue: f64, thresholds: &Vec<f64>) -> usize {
    let mut index: usize = 0;
    for &threshold in thresholds.iter() {
        if hue >= threshold {
            index += 1;
        } else {
            break;
        }
    }
    index
}

fn luminance(pixel: &Rgb<TSrgb, u8>) -> f64 {
    (0.212671 * pixel.red as f64 + 0.715160 * pixel.green as f64 + 0.072169 * pixel.blue as f64)
        / 255.
}

#[derive(Parser)]
struct Cli {
    input: String,
    #[arg(long, short = 'h', value_parser, num_args=0.., value_delimiter = ',')]
    hue_thresholds: Vec<f64>,
    #[arg(long, short = 's')]
    saturation_chunks: usize,
    #[arg(long, short = 'l')]
    luminance_chunks: Option<usize>,
    #[arg(long, short = 'v')]
    value_chunks: Option<usize>,
    output: String,
}
