# Denoise-cli

A command-line utility for running a multichannel denoising algorithm (from [image-recovery](https://docs.rs/image-recovery/latest/image_recovery/)).

## How to use:

You can check the necessary input parameters at any time by running:

`denoise-cli --help`

Basically, you need to supply:
- `-i` an [i]nput image,
- `-o` the directory where you want the [o]utput images to be,
- `-s` a [s]tarting value for `λ`,
- `-e` an [e]nding value for `λ`,
- `-t` how many values of `λ` should be used (s[t]eps),
- `-m` the [m]aximum amount of iterations to run for each value of `λ`,
- `-c` the [c]onvergence threshold for exiting the algorithm.

The program will try to detect the available parallelism to run the denoising for each value of `λ` in a separate thread. By default it will spawn as many threads as there the available parallelism, but you may supply a maximum:
- `--max-parallelism` a non zero integer for the maximum threads to spawn.

Optionally you may supply the verbosity level of the output:
- `-v` for WARN,
- `-vv` for INFO,
- `-vvv` for DEBUG,
- `-vvvv` for TRACE,

You can do that like so:

`denoise-cli -vv -i angry_birb_noisy.png -o . -s 0.001 -e 0.08 -t 20 -m 1000 -c 10e-10`

- This will produce 20 images, the first using `λ = 0.001` and the last using `λ = 0.08`, with the images in between using intermediary values of `λ` spread geometrically.

Have fun! :sparkles:

## Example:

Running:

`./denoise-cli -i birb_noisy.png -o . -s 0.001 -e 0.08 -t 5 -m 1000 -c 10e-10`

Results in:

|λ = 0.0010000000|λ = 0.0029906976|λ = 0.0089442719|λ = 0.0267496122|λ = 0.0800000000|
|---|---|---|---|---|
|![Denoised image with λ = 0.0010000000](https://imgur.com/BO0iGTk.png)|![Denoised image with λ = 0.0029906976](https://imgur.com/OS0yUbv.png)|![Denoised image with λ = 0.0089442719](https://imgur.com/3ByU8xj.png)|![Denoised image with λ = 0.0267496122](https://imgur.com/KN2lyRT.png)|![Denoised image with λ = 0.0800000000](https://imgur.com/EDoFNud.png)|

Input image source: [birb_noisy.png](https://imgur.com/amvPNoJ) by Markus S. Juvonen under License [CC-BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). Gaussian noise was added to the original image using GIMP.

Resulting images found [here](https://imgur.com/a/PISh6XQ). Copyright by Markus S. Juvonen under License [CC-BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). Gaussian noise was added to the original image using GIMP; Noisy image was denoised using the denoising algorithm from the [image-recovery library](https://docs.rs/image-recovery/0.1.0/image_recovery/).

## Copyright

This code is licensed under the GNU Affero General Public License version 3 or later. See [LICENSE](LICENSE) or https://www.gnu.org/licenses/agpl-3.0.en.html.

## Acknowledgements

This is a CLI wrapper for the denoising algorithm in the library [image-recovery](https://github.com/lily-mosquitoes/image-recovery): code by [Lílian Ferreira de Freitas](https://github.com/lily-mosquitoes),
mathematics by [Emilia L. K. Blåsten](https://orcid.org/0000-0001-6675-6108)
