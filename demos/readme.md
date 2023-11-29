# Demos

In this directory, we provide two apps that are used to verify, profile, and benchmark the FPT-accelerated execution of `tfhe-rs`.

:warning: This demo version of FPT implements only a toy parameter set. The goal is to illustrate that FPT is available on AWS, functional, and integrated with `tfhe-rs`. The full design is much more capable, as described in the paper. If you have an application that could benefit from full parameter set FPT acceleration, please [send us an email](mailto:michiel.vanbeirendonck@esat.kuleuven.be,janpieter.danvers@esat.kuleuven.be,furkan.turan@esat.kuleuven.be,ingrid.verbauwhede@esat.kuleuven.be).


## Setup Before Running the Demos

To run the demos, you must first configure the AWS F1 instances. This includes installing AWS and AMD-provided drivers. A guide [`aws_f1_setup.md`](aws_f1_setup.md) is provided to describe the setup in steps.

Let's clone this repository:

```bash
FPT_DIR=~/fpt-demo
git clone https://github.com/KULeuven-COSIC/fpt-demo.git $FPT_DIR
cd $FPT_DIR
```

You should set these two environmental variables, which will indicate to the demo where to find the FPGA image of the FPT.

```bash
export FPGA_IMAGE="$FPT_DIR/demos/accel.awsxclbin"
export FPGA_INDEX=0
```

:warning: In AWS F1 context, the `accel.awsxclbin` is a container that does not contain the actual FPGA image, but rather links to the image hosted by the AWS. It also makes the image subject to access control. If you wish to run this demo, send us an email with your AWS user ID, and we will grant you access to the image.

Once the setup above is done, you can continue with the demos below.

## Demo: FPGA Test

Once AWS F1 instance is set up, this app tests the correctness of FPT. It also allows benchmarking performance etc. 

```bash
cargo run --release --bin fpga --features fpga
```

## Demo: Game-of-Life w/o FPGA

[/demos/game-of-life/readme.md](/demos/game-of-life/readme.md)

To run the demo *without* FPT acceleration:
```bash
cargo run --release --bin game-of-life
```

To run the demo *with* FPT acceleration:
```bash
cargo run --release --bin game-of-life --features fpga
```
