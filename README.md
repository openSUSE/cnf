# Command not found handler for openSUSE

 1. A replacement for a [python based command-not-found handler](
https://github.com/openSUSE/scout/blob/master/handlers/bin/command-not-found) for openSUSE.
 2. Written in Rust, so has as little runtime dependencies as possible.
 3. Uses [libsolv](https://github.com/openSUSE/libsolv) under the hood, so is 100% compatible
    with a zypper.

## Differences

|   | cnf  | scout(cnf)   |
|---|---------|-------|
|Uses | `libsolv` | `libsov` |
|Written in| Rust | shell, two packages in Python |
|Detect enabled/disabled repos? | Yes | Yes |
|Tries to refresh repos| No | Yes |
|bash integration | Yes | Yes|
|zsh integration | Yes | Yes |
|turn off integration | uninstall package | magic variable |
|localization | Yes (UTF-8) only | Yes |

## Build

```sh
cargo build
```

## Direct usage

To query not installed programs

```
./target/debug/cnf cmake

The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

To query installed programs in `/usr/sbin`

```
./target/debug/cnf chcpu
Absolute path to 'chcpu' is '/usr/sbin/chcpu', so running it may require superuser privileges (eg. root).
```

To query installed programs in `/usr/bin`

```
./target/debug/cnf vim  
Absolute path to 'vim' is '/usr/bin/vim'. Please check your $PATH variable to see whether it contains the mentioned path
```

##  Integrate with bash

```sh
> source command_not_found_bash
> export COMMAND_NOT_FOUND_BIN=./target/debug/cnf
> cmake

The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

## Integration tests

Integration tests runs inside docker image tagged `local/cnf-ci`. It is
built as a part of Github Action and can be built locally as `cd test; docker
build -t local/cnf-ci:latest .`

The testing itself is wrapped in [bats](https://github.com/bats-core/bats-core)
and in order to make it run, one needs to initialize the git submodules (`git
submodule init`). Then tests can be executed using a following command

```sh
$ ./test/bats/bin/bats ./test/
test.bats
 ✓ root: installed /usr/bin/rpm
 ✓ root: installed /usr/sbin/sysctl
 ✓ root: not installed single package
 ✓ root: not installed more packages
 ✓ root: bash handler: not installed more packages
 ✓ nonroot: not installed more packages

6 tests, 0 failures

```

Every test can be executed on a command line. The `root.sh` wrapper mounts the
binary to `/usr/bin/cnf` and mounts the `libsolv.so.1` if running on ubuntu-amd64
or if shared library is in `test/libsolv.so.1`. This is done in order to solve
the packaging difference of a libsolv between openSUSE and Ubuntu.

```sh
$ ./root.sh /usr/bin/cnf rpm
Absolute path to 'rpm' is '/usr/bin/rpm'. Please check your $PATH variable to see whether it contains the mentioned path
```

## Powershell users

As cnf does not integrate with a power shell by default, please follow issue comment https://github.com/openSUSE/cnf/issues/8#issuecomment-1854638267
to learn how to configure your system properly.
