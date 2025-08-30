setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'

    bats_require_minimum_version 1.11.0

    # get the containing directory of this file
    # use $BATS_TEST_FILENAME instead of ${BASH_SOURCE[0]} or $0,
    # as those will point to the bats executable's location or the preprocessed file respectively
    DIR="$( cd "$( dirname "$BATS_TEST_FILENAME" )" >/dev/null 2>&1 && pwd )"
    # make executables in test/ visible to PATH
    PATH="$DIR/../test:$PATH"
}

last_id=0
# Dynamically make a test: the description is $1, and the body is stdin
function make_test {
    name=test_$last_id
    last_id=$((last_id + 1))
    eval function "$name" $'{\n' $(cat) $'\n}'
    bats_test_function --description "$1" -- "$name"
}

for PM in zypper dnf5; do

make_test "$PM root: installed /usr/bin/rpm" <<EOF
    run root.sh $PM '/usr/bin/cnf' 'rpm'
    assert_output --partial "Absolute path to 'rpm' is '/usr/bin/rpm'. Please check your \$PATH variable to see whether it contains the mentioned path."
EOF

make_test "$PM root: installed /usr/sbin/sysctl" <<EOF
    run root.sh $PM '/usr/bin/cnf' 'sysctl'
    assert_output --partial "Absolute path to 'sysctl' is '/usr/sbin/sysctl', so running it may require superuser privileges (eg. root)."
EOF

make_test "$PM root: not installed xnake" <<EOF
    run -127 root.sh $PM '/usr/bin/cnf' 'xnake'
    assert_output --partial " xnake: command not found"
EOF

make_test "$PM root: not installed make" <<EOF
    run root.sh $PM '/usr/bin/cnf' 'make'
    assert_output --partial "The program 'make' can be found in the following package:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM root: not installed cmake" <<EOF
    run root.sh $PM '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM nonroot: not installed cmake" <<EOF
    run nonroot.sh $PM '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM nonroot: bash without handler: not installed cmake" <<EOF
    run -127 nonroot.sh $PM bash -c "cmake"
    assert_output --partial "bash: line 1: cmake: command not found"
EOF

make_test "$PM nonroot: bash handler: not installed cmake" <<EOF
    run -127 nonroot.sh $PM bash -c "source /usr/etc/bash_command_not_found; cmake"
    assert_output --partial "The program 'cmake' can be found in following packages:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM nonroot: zsh without handler: not installed cmake" <<EOF
    run -127 nonroot.sh $PM zsh -c "cmake"
    assert_output --partial "zsh:1: command not found: cmake"
EOF

make_test "$PM nonroot: zsh handler: not installed cmake" <<EOF
    run -0 nonroot.sh $PM zsh -c "source /usr/etc/zsh_command_not_found; cmake"
    assert_output --partial "The program 'cmake' can be found in following packages:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM nonroot: fish handler: not installed cmake" <<EOF
    run -127 nonroot.sh $PM fish -c 'source /usr/share/fish/functions/fish_command_not_found.fish; cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
    assert_output --partial "sudo $PM install"
EOF

make_test "$PM issue26: do not list not installable files" <<EOF
    run root.sh $PM '/usr/bin/cnf' 'fractal'
    assert_output --partial "sudo $PM install"
    pkg_lines="$(printf '%s\n' "$output" | grep -c -E '^  \*')"
    [ "$pkg_lines" -eq 2 ]
EOF

# TODO: install i18n

done
