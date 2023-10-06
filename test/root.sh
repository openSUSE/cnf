#!/bin/bash

set -e

#
# Runs cnf binary inside (openSUSE) container. Mounts the cnf, bash integration and
# libsolv.so.1 into container ensuring the tool can run there.
#
# To be executed manually or via bats test.bats
#

PROJECT_DIR=$(readlink -f "$( dirname "$BATS_TEST_FILENAME" )/..")
CNF_SRC=$(readlink -f "${PROJECT_DIR}/target/debug/cnf")
BASH_CNF=$(readlink -f "${PROJECT_DIR}/command_not_found_bash")

USER="root"
if [[ "${0}" =~ "nonroot.sh" ]]; then
    USER="nonroot"
fi

VOLUMES=()
VOLUMES[0]="--volume "${CNF_SRC}":/usr/bin/cnf:ro"
# needed for fish shell: boo#1215428
VOLUMES[1]="--volume "${CNF_SRC}":/usr/bin/command-not-found:ro"
VOLUMES[2]="--volume "${BASH_CNF}":/etc/bash_command_not_found:ro"

# github action uses the ubuntu-latest with libsolv-dev, which is a shared library
# openSUSE distributes libsolv only as static library on the other hand
# so if running the script on ubuntu-latest, mount the shared library into openSUSE container
# to make it run there
if [[ -f /usr/lib/x86_64-linux-gnu/libsolv.so.1 ]]; then
    VOLUMES[3]="--volume /usr/lib/x86_64-linux-gnu/libsolv.so.1:/usr/lib64/libsolv.so.1:ro"
elif [[ -f "${PROJECT_DIR}/test/libsolv.so.1" ]]; then
    # mount the libsolv.so.1 if it is in test/ directory: this supports the development
    VOLUMES[3]="--volume "${PROJECT_DIR}/test/libsolv.so.1":/usr/lib64/libsolv.so.1:ro"
fi

docker \
    run \
    --rm \
    --user "${USER}" \
    ${VOLUMES[*]} \
    local/cnf-ci:latest "${@}"
