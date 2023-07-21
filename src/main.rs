// Copyright (C) 2022  Lílian Ferreira de Freitas
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![feature(path_file_prefix)]

use std::{
    path::PathBuf,
    thread,
};

use clap::{
    CommandFactory,
    Parser,
};
use image_recovery::{
    image,
    img::Manipulation,
    solvers,
    RgbMatrices,
};

/// CLI wrapper for the denoising algorithm from image-recovery.
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
#[command(author, version, about)]
struct Cli {
    /// Path of input image
    #[arg(short, long)]
    input_image: PathBuf,
    /// Path of folder in which output images should be saved
    #[arg(short, long)]
    output_folder: PathBuf,
    /// Maximum number of iterations
    #[arg(short, long)]
    max_iter: u32,
    /// Convergence threshold
    #[arg(short, long)]
    convergence_threshold: f64,
    /// Starting range for lambda values
    #[arg(short = 's', long)]
    start_lambda: f64,
    /// End range for lambda values
    #[arg(short = 'e', long)]
    end_lambda: f64,
    /// Number of steps, i.e. lambda values to use;
    /// Cannot be zero. `-t=1` will produce a single output
    /// using the --start-lambda value
    #[arg(short = 't', long)]
    steps: std::num::NonZeroUsize,
    /// Verbosity (from -v to -vvvv)
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        value_parser = clap::value_parser!(u8).range(..=4),
    )]
    verbose: u8,
}

fn validate_args(args: &Cli) {
    let mut cmd = Cli::command();

    if !args.input_image.is_file() {
        cmd.error(
            clap::error::ErrorKind::ValueValidation,
            "`input_image` must bet a valid file",
        )
        .exit();
    }

    if !args.output_folder.is_dir() {
        cmd.error(
            clap::error::ErrorKind::ValueValidation,
            "`output_path` must be a valid directory",
        )
        .exit();
    }

    if !(args.start_lambda < args.end_lambda) {
        cmd.error(
            clap::error::ErrorKind::ValueValidation,
            "`start_lambda` must be smaller than `end_lambda`",
        )
        .exit();
    }
}

fn main() {
    let args = Cli::parse();
    validate_args(&args);

    let verbosity = match args.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    Logger::init_with_level_filter(verbosity).unwrap();
    log::trace!("log level is TRACE");

    let img = image::open(&args.input_image)
        .expect("image could not be open")
        .into_rgb8();

    // load the RGB image into an object which is composed
    // of 3 matrices, one for each channel
    let img_matrices = img.to_matrices();

    // calculate `q`, the multiplier for the number of steps
    let q = (args.end_lambda / args.start_lambda)
        .powf(1_f64 / (args.steps.get() - 1) as f64);

    // calculate the lambda(s) to use
    let lambdas = (0..args.steps.get())
        .map(|step| &args.start_lambda * q.powi(step as i32));

    let make_output_path_for = |lambda: f64| -> PathBuf {
        let file_name = format!(
            "{}_lambda_=_{:.10}.png",
            args.input_image
                .file_prefix()
                .unwrap_or(std::ffi::OsStr::new("img"))
                .to_string_lossy(),
            lambda
        );
        let mut output_path = args.output_folder.clone();
        output_path.push(file_name);
        log::info!("set output file name: {}", output_path.to_string_lossy());
        output_path
    };

    match thread::available_parallelism() {
        Ok(_) => {
            let mut handles = Vec::with_capacity(lambdas.len());
            for lambda in lambdas {
                let img_matrices = img_matrices.clone();
                let output_path = make_output_path_for(lambda);
                handles.push((
                    lambda,
                    thread::spawn(move || {
                        log::debug!(
                            "spawned thread for lambda: {:.10}",
                            lambda
                        );
                        denoise_and_save(
                            &img_matrices,
                            args.max_iter,
                            args.convergence_threshold,
                            lambda,
                            &output_path,
                        );
                    }),
                ));
            }
            for (lambda, handle) in handles {
                log::debug!("calling join on thread for lambda: {}", lambda);
                handle.join().expect(&format!(
                    "thread of lambda {} has panicked",
                    lambda
                ));
            }
        },
        Err(message) => {
            log::warn!("no available parallelism: {}", message);
            for lambda in lambdas {
                let output_path = make_output_path_for(lambda);
                denoise_and_save(
                    &img_matrices,
                    args.max_iter,
                    args.convergence_threshold,
                    lambda,
                    &output_path,
                );
            }
        },
    };
}

fn denoise_and_save(
    image: &RgbMatrices,
    max_iter: u32,
    convergence_threshold: f64,
    lambda: f64,
    output_file_name: &PathBuf,
) {
    // choose tau and sigma inputs for the denoising solver:
    // according to Chambolle, A. and Pock, T. (2011),
    // tau and lambda should be chosen such that
    // `tau * lambda * L2 norm^2 <= 1`
    // while `L2 norm^2 <= 8`
    // If we choose `tau * lambda * L2 norm^2 == 1`, then:
    let tau: f64 = 1.0 / 2_f64.sqrt();
    let sigma: f64 = 1_f64 / (8.0 * tau);

    // gamma is a variable used to update the internal
    // state of the algorithm's variables, providing
    // an accelerated method for convergence.
    // Chambolle, A. and Pock, T. (2011), choose
    // the value to be `0.35 * lambda`
    let gamma: f64 = 0.35 * lambda;

    // now we can call the denoising solver with the chosen variables
    let denoised = solvers::denoise_multichannel(
        image,
        lambda,
        tau,
        sigma,
        gamma,
        max_iter,
        convergence_threshold,
    );

    // we convert the solution into an RGB image format
    let new_img = image::RgbImage::from_matrices(&denoised);

    // encode it and save it to a file
    new_img
        .save(output_file_name)
        .expect("image could not be saved");
    log::info!("image saved: {}", output_file_name.to_string_lossy());
}

static LOGGER: Logger = Logger;

struct Logger;

impl Logger {
    fn init_with_level_filter(
        log_level: log::LevelFilter,
    ) -> Result<(), log::SetLoggerError> {
        log::set_logger(&LOGGER)?;
        log::set_max_level(log_level);
        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        match log::max_level().to_level() {
            Some(level) => metadata.level() <= level,
            None => false,
        }
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                log::Level::Error => {
                    eprintln!("{}: {}", log::Level::Error, record.args());
                },
                level => {
                    println!("{}: {}", level, record.args());
                },
            }
        }
    }

    fn flush(&self) {
        unimplemented!();
    }
}
