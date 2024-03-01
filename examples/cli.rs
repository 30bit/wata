use std::{
    ffi::OsStr,
    fs::{self, File},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use wata::WriteConfig;
use zip::ZipArchive;

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Make and write `wata` file
    Make {
        /// Path to `h264`-encoded video
        input: PathBuf,
        #[arg(long, short = 'w')]
        frame_width: u32,
        #[arg(long, short = 'l')]
        frame_height: u32,
        #[arg(long, short = 's')]
        is_srgb: Option<bool>,
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract internals from `wata` file
    Extract {
        /// Path to `wata` file
        input: PathBuf,
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    match Args::parse().command {
        Commands::Make {
            input: input_path,
            frame_width,
            frame_height,
            is_srgb,
            output: output_path,
        } => {
            let h264_buf = fs::read(&input_path)?;
            let img = wata::make(frame_width, frame_height, &h264_buf)?;
            let output_path = output_path.unwrap_or_else(|| {
                let mut output_path = input_path;
                output_path.set_extension("wata");
                output_path
            });
            let output_file = File::create(output_path)?;
            wata::write(
                output_file,
                &img,
                &WriteConfig {
                    frame_height,
                    is_srgb,
                },
            )?;
        }
        Commands::Extract {
            input: input_path,
            output: output_path,
        } => {
            let mut input_file = ZipArchive::new(File::open(&input_path)?)?;
            let output_path = output_path.unwrap_or_else(|| {
                let mut output_path = input_path;
                output_path.set_extension("");
                let file_name = output_path
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("output"))
                    .to_owned();
                output_path.pop();
                output_path.push(file_name);
                output_path
            });
            input_file.extract(output_path)?;
        }
    }
    Ok(())
}
