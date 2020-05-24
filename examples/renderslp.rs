use anyhow::bail;
use genie::slp::{Command, RGBA8};
use genie::{Palette, SLP};
use image::png::PNGEncoder;
use image::{ColorType, ImageResult};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    filename: PathBuf,
    #[structopt(long = "out", short = "o")]
    output: PathBuf,
    #[structopt(long = "palette", short = "p")]
    palette: Option<PathBuf>,
    #[structopt(long = "frame", short = "f")]
    frame: Option<usize>,
}

struct Output {
    f: File,
    size: (u32, u32),
    image_data: Vec<u8>,
}

impl Output {
    fn new(path: impl AsRef<Path>, size: (u32, u32)) -> io::Result<Self> {
        let f = File::create(path)?;

        Ok(Self {
            f,
            size,
            image_data: Vec::with_capacity((size.0 * size.1 * 4) as usize),
        })
    }

    fn write_pixel(&mut self, pixel: RGBA8) {
        // println!("write {:?}", pixel);
        let RGBA8 { r, g, b, a } = pixel;
        self.image_data.extend_from_slice(&[r, g, b, a]);
    }

    fn write(&mut self, command: Command<RGBA8>) {
        match command {
            Command::Copy(pixels) => {
                pixels.into_iter().for_each(|p| self.write_pixel(p));
            }
            Command::Fill(num, pixel) => {
                (0..num).for_each(|_| self.write_pixel(pixel));
            }
            Command::PlayerCopy(pixels) => todo!(),
            Command::PlayerFill(num, pixel) => todo!(),
            Command::Skip(num) => {
                (0..num).for_each(|_| self.write_pixel(RGBA8::default()));
            }
            Command::NextLine => {
                // should do this differently
                let pixel_width = self.size.0;
                let byte_width = (pixel_width * 4) as usize;
                let line_progress = self.image_data.len() % byte_width;
                if line_progress == 0 {
                    return;
                }
                let byte_remaining = byte_width - line_progress;
                let pixel_remaining = byte_remaining / 4;
                (0..pixel_remaining).for_each(|_| self.write_pixel(RGBA8::default()));
            }
        }
    }

    fn finish(self) -> ImageResult<()> {
        let encoder = PNGEncoder::new(self.f);
        encoder.encode(&self.image_data, self.size.0, self.size.1, ColorType::Rgba8)?;
        Ok(())
    }
}

pub fn main() -> anyhow::Result<()> {
    let args = Cli::from_args();
    let f = File::open(&args.filename)?;
    let slp = SLP::read_from(f)?;

    if let Some(frame) = args.frame {
        let frame = slp.frame(frame);
        if frame.is_8bit() && args.palette.is_none() {
            bail!("That frame uses 8-bit palette indexes. Please provide a `--palette` file");
        }
        if frame.is_32bit() && args.palette.is_some() {
            println!("note: Ignoring palette because the frame uses 32-bit colour");
        }

        let mut output = Output::new(args.output, frame.size())?;
        if frame.is_8bit() {
            let palette = {
                let f = File::open(args.palette.unwrap())?;
                Palette::read_from(f)?
            };

            for command in frame.commands_8bit() {
                let command = command?.map_color(|index| palette[index].alpha(255));
                output.write(command);
            }
        } else {
            for command in frame.commands_32bit() {
                let command = command?;
                output.write(command);
            }
        }

        output.finish()?;
    } else {
        for (id, frame) in slp.frames().enumerate() {
            println!("#{} - {}×{}", id, frame.size().0, frame.size().1);
        }
    }

    Ok(())
}
