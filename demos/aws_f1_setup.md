# Setting up the F1 Instance

## Creating an AWS account

:exclamation: A new AWS account may not have the required quota allowance to run a `f1.2xlarge` type instance. If that is the case, one should file a [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service. Make sure to request a limit increase to `8 instances`, since `f1.2xlarge` contains 8 vCPUs. The quota increase may take up to a few days to process.

In your communication to AWS, please pay attention that the F1 access permissions are tied to a given region. Though we provide the demo application in this repository, the actual FPGA image is hosted by AWS. We make that image publicly available in all the F1 instance regions of today, which are `us-east-1`, `us-west-2`, `eu-west-1`, `ap-southeast-2`, `eu-central-1` and `eu-west-2`. If more regions with F1 instances appear in future, we will make the image available in those regions too. If you notice that we are late to do this, you can create an issue.

:exclamation: In fact, we also prepared the instructions in [aws_afi_gen.md](./aws_afi_gen.md), so that you can create the image yourself from the actual image file (`.xclbin`); however, the file is 620MB (when compressed 570MB), hence exceeds the file size limitations of GitHub. We will look for a solution to share it in another way.

## Instantiate an F1 Instance

Instantiate an `f1.2xlarge` instance with `FPGA Developer AMI`. This AMI is not free of charge, but it is easier to work with for short-time uses of this demo. Otherwise, the XRT installation is actually more complicated than described [here](https://github.com/aws/aws-fpga/blob/master/Vitis/docs/XRT_installation_instructions.md).

Please note that, the instructions below are NOT prepared for the `Amazon Linux 2` version, or the one from `Community AMI`.

## Preparation

Once the instance is initialised, login and install the dependencies.

At first, we should update the package repo URLs. Unfortunately, `FPGA Developer AMI` is based on CentOS 7, which is arriving to its End-of-Life. Its default URL to package download mirror is already obsolate. As a workaround, we will start with updating the `.repo` files:

```bash
sudo sed -i s/mirror.centos.org/vault.centos.org/g /etc/yum.repos.d/*.repo
sudo sed -i s/^#.*baseurl=http/baseurl=http/g /etc/yum.repos.d/*.repo
sudo sed -i s/^mirrorlist=http/#mirrorlist=http/g /etc/yum.repos.d/*.repo
```
Note that: we might need to re-run these commands, if the following `yum` commands fail; one can introduce a new `yum.repo`, causing the next one to fail.

Let's start with the basics:
```bash
sudo yum update
sudo yum upgrade
sudo yum group install "Development Tools"
```
___

Next, we will install Rust for compiling `tfhe-rs`. 

First, we should install a few dependencies.

Install `alsa`:
```bash
sudo yum install alsa-lib-devel
```

Install `systemd-devel`:
```bash
sudo yum install systemd-devel
```

This one is to install a newer version of `clang`, the default of which is lower than 3.5, and not compatible:
```bash
sudo yum install centos-release-scl
sudo yum install llvm-toolset-7
scl enable llvm-toolset-7 bash
```

Now, install Rust:
```bash
curl https://sh.rustup.rs -sSf > RUSTUP.sh
sh RUSTUP.sh -y
rm RUSTUP.sh
source "$HOME/.cargo/env"
```

The demo uses the `nightly` version of Rust, and preferrably the `2023-11-29` version used at the preparation of this demo:
```bash
rustup install nightly-2023-11-29
rustup default nightly-2023-11-29
```
___

For preparing the FPGA runtime, we will need the `aws-fpga` repo:
```bash
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
