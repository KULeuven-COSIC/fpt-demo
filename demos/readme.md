# Demos

In this directory, we provide two apps that are used to verify, profile, and benchmark the FPT accelerated execution of `TFHE-rs`.

:exclamation: Remember that, this demo version of the FPT implements only a reduced parameter set to have the software TFHE-rs board updates in acceptable time. The full design is much more capable, as described in the paper.

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

:exclamation: In AWS F1 context, the `accel.awsxclbin` is a container that does not contain the actual FPGA image, but rather links to the image hosted by the AWS. It also makes the image subject to access control. If you wish to run this demo, send us an email with your AWS user ID, and we will grant you access to the image.

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
