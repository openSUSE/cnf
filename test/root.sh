#!/bin/bash

set -e

#
# Runs cnf binary inside (openSUSE) container. Mounts the cnf, bash and fish
# integration and into container ensuring the tool can run there.
#
# To be executed manually or via bats test.bats
#

PROJECT_DIR=$(readlink -f "$( dirname "$BATS_TEST_FILENAME" )/..")
CNF_SRC=$(readlink -f "${PROJECT_DIR}/target/debug/cnf")
BASH_CNF=$(readlink -f "${PROJECT_DIR}/command_not_found.bash")
ZSH_CNF=$(readlink -f "${PROJECT_DIR}/command_not_found.zsh")

USER="root"
if [[ "${0}" =~ nonroot.sh ]]; then
    USER="nonroot"
fi

VOLUMES=()
VOLUMES[0]="--volume ${CNF_SRC}:/usr/bin/cnf:ro"
# needed for fish shell: boo#1215428
VOLUMES[1]="--volume ${CNF_SRC}:/usr/bin/command-not-found:ro"
VOLUMES[2]="--volume ${BASH_CNF}:/usr/etc/bash_command_not_found:ro"
VOLUMES[3]="--volume ${ZSH_CNF}:/usr/etc/zsh_command_not_found:ro"

PM=$1
shift
docker \
    run \
    --tty \
    --rm \
    --user "${USER}" \
    ${VOLUMES[*]} \
    local/cnf-ci-$PM:latest "${@}"