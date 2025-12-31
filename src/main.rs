use clap::Parser;
use image::{ImageBuffer, Luma};
use std::{fs::{self, DirEntry}, path::PathBuf};
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

struct PathSimilarity {
    path: PathBuf,
    similarity: f64
}

struct FrameFromPool {
    source_frame: PathBuf,
    pool_frame: PathBuf
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

    input_frames.par_iter()
        .map(|f| {
            println!("Handling {}", f.dir_entry.file_name().to_string_lossy());
            FrameFromPool {
                source_frame: f.dir_entry.path(),
                pool_frame: pool.par_iter()
                    .map(
                        |p|
                        PathSimilarity {
                            path: p.dir_entry.path(),
                            similarity: image_compare::gray_similarity_structure(
                                &image_compare::Algorithm::RootMeanSquared, &f.buffer, &p.buffer
                            )
                            .expect("Error comparing images.")
                            .score
                        }
                    )
                    .max_by(|x, y| x.similarity.abs().total_cmp(&y.similarity.abs()))
                    .unwrap()
                    .path
            }
        })
        .for_each(|r| {
            let _ = fs::copy(r.pool_frame, args.out_dir.join(r.source_frame.file_name().unwrap()));
        });

    println!("Done!");
}
