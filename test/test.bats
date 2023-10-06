setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'

    # get the containing directory of this file
    # use $BATS_TEST_FILENAME instead of ${BASH_SOURCE[0]} or $0,
    # as those will point to the bats executable's location or the preprocessed file respectively
    DIR="$( cd "$( dirname "$BATS_TEST_FILENAME" )" >/dev/null 2>&1 && pwd )"
    # make executables in test/ visible to PATH
    PATH="$DIR/../test:$PATH"
}

@test "root: installed /usr/bin/rpm" {
    run root.sh '/usr/bin/cnf' 'rpm'
    assert_output "Absolute path to 'rpm' is '/usr/bin/rpm'. Please check your \$PATH variable to see whether it contains the mentioned path."
}

@test "root: installed /usr/sbin/sysctl" {
    run root.sh '/usr/bin/cnf' 'sysctl'
    assert_output "Absolute path to 'sysctl' is '/usr/sbin/sysctl', so running it may require superuser privileges (eg. root)."
}


@test "root: not installed single package" {
    run root.sh '/usr/bin/cnf' 'make'
    assert_output --partial "The program 'make' can be found in the following package:"
}

@test "root: not installed more packages" {
    run root.sh '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "root: bash handler: not installed more packages" {
    run root.sh '/usr/bin/cnf' 'cmake'
    root.sh bash -c "source /etc/bash_command_not_found; cmake"
    assert_output --partial "The program 'cmake' can be found in following packages:"
}

@test "nonroot: not installed more packages" {
    run nonroot.sh '/usr/bin/cnf' 'cmake'
    assert_output --partial "The program 'cmake' can be found in following packages:"
}
