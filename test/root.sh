#!/bin/bash

set -e

#
# Runs cnf-rs binary inside (openSUSE) container. Mounts the cnf-rs, bash integration and
# libsolv.so.1 into container ensuring the tool can run there.
#
# To be executed manually or via bats test.bats
#

PROJECT_DIR=$(readlink -f "$( dirname "$BATS_TEST_FILENAME" )/..")
CNF_RS_SRC=$(readlink -f "${PROJECT_DIR}/target/debug/cnf-rs")
BASH_CNF=$(readlink -f "${PROJECT_DIR}/test/cnf-rs.bash")

USER="root"
if [[ "${0}" =~ "nonroot.sh" ]]; then
    USER="nonroot"
fi

VOLUMES=()
VOLUMES[0]="--volume "${CNF_RS_SRC}":/src/cnf-rs:ro"
VOLUMES[1]="--volume "${BASH_CNF}":/src/cnf-rs.bash:ro"

# github action uses the ubuntu-latest with libsolv-dev, which is a shared library
# openSUSE distributes libsolv only as static library on the other hand
# so if running the script on ubuntu-latest, mount the shared library into openSUSE container
# to make it run there
if [[ -f /usr/lib/x86_64-linux-gnu/libsolv.so.1 ]]; then
    VOLUMES[2]="--volume /usr/lib/x86_64-linux-gnu/libsolv.so.1:/usr/lib64/libsolv.so.1:ro"
elif [[ -f "${PROJECT_DIR}/test/libsolv.so.1" ]]; then
    # mount the libsolv.so.1 if it is in test/ directory: this supports the development
    VOLUMES[2]="--volume "${PROJECT_DIR}/test/libsolv.so.1":/usr/lib64/libsolv.so.1:ro"
fi

docker \
    run \
    --rm \
    --user "${USER}" \
    ${VOLUMES[*]} \
    local/cnf-rs-ci:latest "${@}"
