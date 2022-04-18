# Denoise solver

A command-line utility for running a multichannel denoising algorithm (from [image-recovery](https://docs.rs/image/latest/image/)).

## How to use:

You can check the parameter at any time by running:

`denoise-solver --help`

Basically, you need to supply:
- an input image,
- the direcotry where you want the output images to be,
- a starting value for `λ`,
- an ending value for `λ`,
- how many values of `λ` should be tested (steps),
- the maximum amount of iterations to run,
- the convergence threshold for exiting the algorithm.

You can do that like so:

`denoise-solver -i birb_noisy.png -o . -s 0.001 -e 0.08 -t 20 -m 1000 -c 10e-10`

- This will produce 20 images, the first using `λ = 0.001` and the last using `λ = 0.08`, with the images in between using intermediary values of `λ`, in a logarithmic scale.

Have fun! :sparkles:
