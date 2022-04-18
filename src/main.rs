#![feature(path_file_prefix)]

use clap::{Parser, CommandFactory, ErrorKind};
use image_recovery::{
    image,
    img::Manipulation,
    solvers,
};

/// Runner for denoising algorithm.
///
/// λ values:
///
/// The algorithm will run on the given input for as
/// many λ values as given. Simply choose a start and
/// end point, as well as how many steps there should
/// be in between.
///
/// Stopping conditions:
///
/// The algorithm will run for at most `max_iter` number
/// of iterations per λ value, but may stop earlier if the
/// relative differente between the current candidate output
/// and the previous iteration's candidate output becomes
/// smaller than the given value for the `convergence_threshold`
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// Path of input image
    #[clap(short, long)]
    input_image: std::path::PathBuf,
    /// Path of folder in which output images should be saved
    #[clap(short, long)]
    output_folder: std::path::PathBuf,
    /// Maximum number of iterations
    #[clap(short, long)]
    max_iter: u32,
    /// Convergence threshold
    #[clap(short, long)]
    convergence_threshold: f64,
    /// Starting range for lambda values
    #[clap(short = 's', long)]
    start_lambda: f64,
    /// End range for lambda values
    #[clap(short = 'e', long)]
    end_lambda: f64,
    /// Number of steps between the lambda values
    #[clap(short = 't', long)]
    steps: i32,
}

fn main() {
    let mut args = Cli::parse();
    let mut cmd = Cli::command();

    if !args.input_image.is_file() {
        cmd.error(ErrorKind::ValueValidation, "`input_image` must bet a valid file").exit();
    }

    if !args.output_folder.is_dir() {
        cmd.error(ErrorKind::ValueValidation, "`output_path` must be a valid directory").exit();
    }

    if !(args.start_lambda < args.end_lambda) {
        cmd.error(ErrorKind::ValueValidation, "`start_lambda` must be smaller than `end_lambda`").exit();
    }

    if !(args.steps > 0) {
        cmd.error(ErrorKind::ValueValidation, "`steps` must be bigger than 0").exit();
    }

    let img = image::open(&args.input_image)
            .expect("image could not be open")
            .into_rgb8();

    // load the RGB image into an object which is composed
    // of 3 matrices, one for each channel
    let img_matrices = img.to_matrices();

    // choose inputs for the denoising solver:
    // according to Chambolle, A. and Pock, T. (2011),
    // tau and lambda should be chosen such that
    // `tau * lambda * L2 norm^2 <= 1`
    // while `L2 norm^2 <= 8`
    // If we choose `tau * lambda * L2 norm^2 == 1`, then:
    let tau: f64 = 1.0 / 2_f64.sqrt();
    let sigma: f64 = 1_f64 / (8.0 * tau);

    // calculate `q`, the multiplier for the number of steps
    let q = (args.end_lambda / args.start_lambda)
        .powf(1_f64 / (args.steps-1) as f64);

    for step in 0..args.steps {
        // calculate the lambda to use
        let lambda = args.start_lambda * q.powi(step);

        // gamma is a variable used to update the internal
        // state of the algorithm's variables, providing
        // an accelerated method for convergence.
        // Chambolle, A. and Pock, T. (2011), choose
        // the value to be `0.35 * lambda`
        let gamma: f64 = 0.35 * lambda;

        // now we can call the denoising solver with the chosen variables
        let denoised = solvers::denoise_multichannel(&img_matrices, lambda, tau, sigma, gamma, args.max_iter, args.convergence_threshold);

        // we convert the solution into an RGB image format
        let new_img = image::RgbImage::from_matrices(&denoised);

        // encode it and save it to a file
        let file_name = format!("{}_lambda_=_{:.10}.png", args.input_image.file_prefix().unwrap().to_str().unwrap(), lambda);
        args.output_folder.set_file_name(file_name);

        new_img.save(&args.output_folder)
            .expect("image could not be saved");
    }
}
