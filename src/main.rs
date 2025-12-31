use clap::Parser;
use image::{ImageBuffer, Luma};
use image_compare::Algorithm;
use std::{fs::DirEntry, path::PathBuf};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    in_dir: std::path::PathBuf,
    #[arg(short, long)]
    frame_pool_dir: std::path::PathBuf,
    #[arg(short, long)]
    out_dir: std::path::PathBuf
}

struct Image {
    buffer: ImageBuffer<Luma<u8>, Vec<u8>>,
    dir_entry: DirEntry
}

fn read_images(path: PathBuf) -> Vec<Image> {
    println!("Reading input files...");
    let total_input_frames = path.read_dir().unwrap().count();
    println!("Total frames: {}", total_input_frames);

    return path.read_dir()
        .expect(format!("Couldn't read {}", path.to_string_lossy()).as_str())
        .map(|r| r.expect("Couldn't read DirEntry"))
        .collect::<Vec<DirEntry>>()
        .into_par_iter().map(|v| {
            let buffer = image::open(v.path())
                .expect(format!("Couldn't open file {}", v.path().to_string_lossy()).as_str())
                .into_luma8();
            return Image { buffer, dir_entry: v }
        })
        .collect();
}

fn main() {
    let args = Args::parse();
    let input_frames = read_images(args.in_dir);
    let pool = read_images(args.frame_pool_dir);

    for frame in input_frames {
        for pool_frame in pool.as_slice() {
            let diff = image_compare::gray_similarity_structure(
                &Algorithm::MSSIMSimple, &frame.buffer, &pool_frame.buffer
            )
            .expect(
                format!(
                    "Mismatch between dimensions of {} and {}",
                    frame.dir_entry.path().to_string_lossy(),
                    pool_frame.dir_entry.path().to_string_lossy()
                ).as_str()
            );
            println!(
                "{} difference between {} and {}",
                diff.score,
                frame.dir_entry.path().to_string_lossy(),
                pool_frame.dir_entry.file_name().to_string_lossy()
            );
        }
    }
}
