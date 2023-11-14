# Setting up the F1 Instance

## Creating an AWS account

:exclamation: A new AWS account may not have the required quota allowance to run a `f1.2xlarge` type instance. If that is the case, one should file a [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service. Make sure to request a limit increase to `8 instances`, since `f1.2xlarge` contains 8 vCPUs. The quota increase may take up to a few days to process.

## Instantiate an F1 Instance

Instantiate an `f1.2xlarge` instance with `FPGA Developer AMI`. (Please note that, the instructions below are NOT prepared for the `Amazon Linux 2` version, or the one from `Community AMI`). This AMI is not free of charge, but it is easier to work with for short-time uses of this demo. Otherwise, the XRT installation is actually more complicated than described [here](https://github.com/aws/aws-fpga/blob/master/Vitis/docs/XRT_installation_instructions.md).

## Preparation

Once the instance is initialised, login and install the dependencies:

Start with the basics:
```
sudo yum update
sudo yum upgrade
sudo yum group install "Development Tools"
```
___

Next, we will install Rust for compiling `tfhe-rs`. 

First, we should install a few dependencies.

Install `alsa`:
```
sudo yum install alsa-lib-devel
```

Install `systemd-devel`:
```
sudo yum install systemd-devel
```

This one is to install a newer version of `clang`, the default of which is lower than 3.5, and not compatible:
```
sudo yum install centos-release-scl
sudo yum install llvm-toolset-7
scl enable llvm-toolset-7 bash
```

Now, install Rust:
```
curl https://sh.rustup.rs -sSf > RUSTUP.sh
sh RUSTUP.sh -y
rm RUSTUP.sh
source "$HOME/.cargo/env"
```

The demo uses the `nightly` version of Rust, hence switch to that version:
```
rustup toolchain install nightly
rustup default nightly
```
___

For preparing the FPGA runtime, we will need the `aws-fpga` repo:
```
AWS_DIR=~/aws-fpga
git clone https://github.com/aws/aws-fpga.git $AWS_DIR
```

## Source the libraries

With the below commands, you need to make the binaries and libraries accessible for compiling the demo application.

```bash
scl enable llvm-toolset-7 bash
source /opt/xilinx/xrt/setup.sh
AWS_DIR=~/aws-fpga
source $AWS_DIR/vitis_runtime_setup.sh
```

Now, you are ready to go. You can follow the instructions on the [readme](./readme.md) file for running the demos.
