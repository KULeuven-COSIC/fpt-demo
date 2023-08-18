# Demos

In this directory we proide two apps that are used to verify, profile and benchmark the FPT accelerated execution of `TFHE-rs`.

:exclamation: Remember that, this demo version of the FPT implements only a reduced parameter set to have the software TFHE-rs board updates in acceptable time. The full design is much more capable as described in the paper.

## Setup Before Running the Demos

To run the demos, you have to first configure the AWS F1 instances, which is mostly installing AWS and AMD provided drivers. A guide [`aws_f1_setup.md`](aws_f1_setup.md) is provided to described the setup in steps.

You should set these two environmental variables, which will indicate the demo the FPGA image of the FPT which will be loaded to the FPGA by the demo code.

```bash
export FPGA_IMAGE="$(pwd)/demos/accel.awsxclbin"
export FPGA_INDEX=0
```

:exclamation: In AWS F1 context, the `accel.awsxclbin` is a container which does not contain the acutal FPGA image, but links to the image hosted by the AWS. It also makes the image subject to access control. If you wish to run this demo, you can drop an email to us with your AWS user ID, and we will grant you access to the image.

Once the setup above are done, you can continue witht the demos below.

## Demo: FPGA Test

Once AWS F1 instance is setup, this app tests the correctness of FPT, besides allows to benchmark performance etc. 

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
