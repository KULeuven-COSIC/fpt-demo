# Creating a new `.awsxclbin` from `.xclbin`

The FPGA image is called `.xclbin`. However, this file cannot be directly used when working on AWS. This file should be submitted to AWS for making it convert it to an AFI (Amazon FPGA Image). This image will be hosted by AWS, and will be accessed by the demo apps through its corresponding `.awsxclbin` file. 

In fact, we already provide an `.awsxclbin` file, which is set publicly accessible on all known AWS regions that employ F1 instances. However, if new regions are introduced in future, you can follow the instructions below, (on an F1 instance), to regenearte a new AFI for that region.

___

Before the start, you should follow the instructions on [aws_f1_setup.md](./aws_f1_setup.md) for setting up an F1 instance. 

You need the version of this repository that includes the `.xclbin` file. As that file is >650 MB, we hosted that version on [Zenodo](https://zenodo.org/), which is the CERN Data Centre-backed research data repository. You can reach to that archive version by [https://zenodo.org/records/10158881](https://zenodo.org/records/10158881).

Let's still work on this GitHub version:

```bash
FPT_DIR=~/fpt-demo
git clone https://github.com/KULeuven-COSIC/fpt-demo.git $FPT_DIR
cd $FPT_DIR
```

Get the `.xclbin` from the Zenodo version:

```
cp <zenodo_version>/demos/accel.xclbin $FPT_DIR/demos/accel.xclbin
```

___

As the first step you should configure the `aws-cli``. Run the command below and provide your credentials and the region of preference. Remember that you should pick the region that has F1 instances.

```bash
aws configure
```

Let's set the environmental variables, in case, they are not already set:

```bash
AWS_REGION=eu-west-1
AWS_DIR=~/aws-fpga
FPT_DIR=~/fpt-demo
FPT_IMAGE="$FPT_DIR/demos/accel.awsxclbin"
XCL_BIN="$FPT_DIR/demos/accel.xclbin"
```

It is preferrable to work a folder within a temporary folder:

```bash
tmp_dir=~/afi-tmp
mkdir -p $tmp_dir
pushd $tmp_dir
```

Create S3 buckets for AWS to place the AFI related files at its generation.

```bash
bucket_name=tfhe-fpga
dcp_key=dcps
logs_key=logs

# Create an S3 bucket
aws s3 mb s3://${bucket_name} --region ${AWS_REGION}

# Create a temp file
# Choose a dcp folder name
touch DCP_FILES_GO_HERE.txt
aws s3 cp DCP_FILES_GO_HERE.txt s3://${bucket_name}/${dcp_key}/
rm -f DCP_FILES_GO_HERE.txt

# Create a temp file
# Choose a logs folder name
touch LOG_FILES_GO_HERE.txt
aws s3 cp LOG_FILES_GO_HERE.txt s3://${bucket_name}/${logs_key}/
rm -f LOG_FILES_GO_HERE.txt
```

Start AFI Creation

```bash
export RELEASE_VER=2021.2

pushd $AWS_DIR
source hdk_setup.sh
popd

$VITIS_DIR/tools/create_vitis_afi.sh -xclbin=${XCL_BIN} -s3_bucket=${bucket_name} -s3_dcp_key=${dcp_key} -s3_logs_key=${logs_key}

# Leave $tmp_dir dir
popd
```

Extract the AFI ID from the created file:

```bash
sudo yum install jq
afi_id=$(jq -r '.FpgaImageId' $tmp_dir/*_afi_id.txt)
echo $afi_id
```

Check the status of the AFI generation. The compilation might take several hours. You can execute the following command to check the status. Until the compilation finishes, the output of this command will indicate `"Code": "pending"`.

```bash
aws ec2 describe-fpga-images --fpga-image-id $afi_id
```

Replace the existing `.awsxclbin` file with the newly created one:

```bash
rm -f $FPT_IMAGE
cp $tmp_dir/*.awsxclbin $FPT_IMAGE
```
