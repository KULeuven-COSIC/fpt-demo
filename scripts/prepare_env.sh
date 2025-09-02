#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$HOME/tfhe-rs"
export TFHERS_COMMIT="2e58fe36a4ac40f9c2a4efb3cccb3d33a67d6439"
export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export PATCH_COMMIT_MSG="FPT Patch Applied"

separator() {
    printf "\n=========================================================\n\n"
}

if [ -d "$TFHERS_DIR" ]; then
    echo "Stash changes and set ZAMA GitHub as the origin"
    pushd $TFHERS_DIR
    git stash -m "Stashed changes"
    git remote set-url origin $TFHERS_URL
else
    echo "Fresh Clone of TFHE-rs"
    git clone --no-checkout $TFHERS_URL $TFHERS_DIR
    pushd $TFHERS_DIR
fi

separator
echo "Checkout TFHE-rs for FPT FPGA acceleration"
PATCH_COMMIT=$(git log --grep="$PATCH_COMMIT_MSG" --format="%H" | head -n 1)

if [ -n "$PATCH_COMMIT" ]; then
    git checkout $PATCH_COMMIT
else
    git checkout $TFHERS_COMMIT

    echo "Applying FPT patch (excluding docs images)..."
    git apply --exclude=tfhe/docs/_static/* --whitespace=nowarn $REPO_DIR/fpt.patch

    separator
    git add .
    echo "Group all changes into one commit"
    git commit -m "$PATCH_COMMIT_MSG"
fi
