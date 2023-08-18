# Installing Libraries and Drivers

## Preparation

The linux kernel heeaders are not enough to build required kernel modules on Ubuntu systems with the most recent kernel versions. Following AWS' suggestion, installing extra kernel modules solves the problem.

```bash
sudo apt update
sudo apt upgrade
sudo apt install rustc cargo libasound2-dev libssl-dev pkg-config clang
sudo apt install linux-virtual
sudo apt install linux-modules-extra-$(uname -r)
sudo apt purge linux*aws
```

At this point, it is advisable to reboot.

```bash
sudo reboot
```

## Clone the XRT (Xilinx Run Time) and aws-fpga repos

In this specific example, XRT's `2021.1` version is used, as the hardware is developed with it. In fact, we could use a newer version, but AWS supports up to `2021.2`, which is buggy.

Let's work with environmental variables.

```bash
export VIVADO_TOOL_VERSION=2021.1
export AWS_DIR=~/aws-fpga
export XRT_DIR=~/xrt
```

Now clone.

```bash
git clone https://github.com/aws/aws-fpga.git $AWS_DIR
git clone https://github.com/Xilinx/XRT.git --branch $VIVADO_TOOL_VERSION $XRT_DIR
```

The compatibility list of `aws-fpga` has only the release version of the XRT. However, that the specific version/commit is also buggy, at least on Ubuntu. Hence, add the most recent version/commit cloned above to the compatibility list.

```bash
pushd $XRT_DIR
hash=$(git rev-parse HEAD)
echo "${VIVADO_TOOL_VERSION}:${hash}" >> ${AWS_DIR}/Vitis/vitis_xrt_version.txt
popd
```

## Installation

Install the XRT dependencies.

```bash
pushd $XRT_DIR
sudo src/runtime_src/tools/scripts/xrtdeps.sh
popd
```

Build XRT.

```bash
pushd $XRT_DIR/build
./build.sh
popd
```

Create `.deb` packages.

```bash
pushd $XRT_DIR/build/Release
make package
popd
```

Install the created `.deb` packages. Here, we preferred to move them to `/tmp` and give their ownership to `_apt`, just to make `apt` happy.

```bash
pushd /tmp
cp $XRT_DIR/build/Release/*.deb .
sudo chown _apt *.deb
sudo chmod 777 *.deb
sudo apt install --reinstall ./*.deb
sudo rm -rf ./*.deb
popd
```

## Source the libraries

Source the scripts below to make the binaries and libraries accesible.

```bash
source /opt/xilinx/xrt/setup.sh
source $AWS_DIR/vitis_runtime_setup.sh
```

## Check the Drivers

At this point you expect the drivers to work, with the following example outputs.

Do not get suruprised, if they do not work :expressionless: Our solution is described below.

```bash
$ systemctl status mpd

System Configuration
  OS Name              : Linux
  Release              : 5.15.0-72-generic
  Version              : #79-Ubuntu SMP Wed Apr 19 08:22:18 UTC 2023
  Machine              : x86_64
  CPU Cores            : 8
  Memory               : 122774 MB
  Distribution         : Ubuntu 22.04.2 LTS
  GLIBC                : 2.35
  Model                : HVM domU

XRT
  Version              : 2.11.0
  Branch               : 2021.1
  Hash                 : 73a310b6e65651f4856a02d23f483883d5517a0f
  Hash Date            : 2023-05-25 07:57:35
  XOCL                 : 2.11.0, 73a310b6e65651f4856a02d23f483883d5517a0f
  XCLMGMT              : 2.11.0, 73a310b6e65651f4856a02d23f483883d5517a0f

Devices present
  [0000:00:1d.0] : xilinx_aws-vu9p-f1_shell-v04261818_201920_2 
```

```bash
$ xbutil examine

System Configuration
  OS Name              : Linux
  Release              : 5.15.0-72-generic
  Version              : #79-Ubuntu SMP Wed Apr 19 08:22:18 UTC 2023
  Machine              : x86_64
  CPU Cores            : 8
  Memory               : 122774 MB
  Distribution         : Ubuntu 22.04.2 LTS
  GLIBC                : 2.35
  Model                : HVM domU

XRT
  Version              : 2.11.0
  Branch               : 2021.1
  Hash                 : 73a310b6e65651f4856a02d23f483883d5517a0f
  Hash Date            : 2023-05-25 07:57:35
  XOCL                 : 2.11.0, 73a310b6e65651f4856a02d23f483883d5517a0f
  XCLMGMT              : 2.11.0, 73a310b6e65651f4856a02d23f483883d5517a0f

Devices present
  [0000:00:1d.0] : xilinx_aws-vu9p-f1_shell-v04261818_201920_2 

```

___

:exclamation: In our experience, the drivers never worked at the first try. Frequently suggested steps (e.g. unmounting and mounting the drivers, restarting the mpd service) also did not help.

Sounds stupid and odd, but we could solve it by repeating the whole **installation** step again.

___

Once, you observe the `xbutil examine` output as given above, you are ready to go. You can follow the instructions on the readme file for running the demos.
