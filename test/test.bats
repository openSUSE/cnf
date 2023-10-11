setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'

    bats_require_minimum_version 1.5.0

    # get the containing directory of this file
    # use $BATS_TEST_FILENAME instead of ${BASH_SOURCE[0]} or $0,
    # as those will point to the bats executable's location or the preprocessed file respectively
    DIR="$( cd "$( dirname "$BATS_TEST_FILENAME" )" >/dev/null 2>&1 && pwd )"
    # make executables in test/ visible to PATH
    PATH="$DIR/../test:$PATH"
}

@test "root: installed /usr/bin/rpm" {
    run root.sh '/usr/bin/cnf' 'rpm'
    assert_output --partial "Absolute path to 'rpm' is '/usr/bin/rpm'. Please check your \$PATH variable to see whether it contains the mentioned path."
}

@test "root: installed /usr/sbin/sysctl" {
    run root.sh '/usr/bin/cnf' 'sysctl'
    assert_output --partial "Absolute path to 'sysctl' is '/usr/sbin/sysctl', so running it may require superuser privileges (eg. root)."
}

@test "root: not installed xnake" {
    run -127 root.sh '/usr/bin/cnf' 'xnake'
    assert_output --partial " xnake: command not found"
}

@test "root: not installed make" {
    run root.sh '/usr/bin/cnf' 'make'
    assert_output --partial "The program 'make' can be found in the following package:"
}

@test "root: not installed cmake" {
    run root.sh '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "nonroot: not installed cmake" {
    run nonroot.sh '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "nonroot: bash without handler: not installed cmake" {
    run -127 nonroot.sh bash -c "cmake"
    assert_output --partial "bash: line 1: cmake: command not found"
}

@test "nonroot: bash handler: not installed cmake" {
    run -127 nonroot.sh bash -c "source /usr/etc/bash_command_not_found; cmake"
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "nonroot: zsh without handler: not installed cmake" {
    run -127 nonroot.sh zsh -c "cmake"
    assert_output --partial "zsh:1: command not found: cmake"
}

@test "nonroot: zsh handler: not installed cmake" {
    run -0 nonroot.sh zsh -c "source /usr/etc/zsh_command_not_found; cmake"
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "nonroot: fish handler: not installed cmake" {
    run -127 nonroot.sh fish -c 'source /usr/share/fish/functions/fish_command_not_found.fish; cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

# TODO: install i18n
