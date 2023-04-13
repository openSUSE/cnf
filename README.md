# Command not found handler for openSUSE

A replacement for a [python based command-not-found handler](
https://github.com/openSUSE/scout/blob/master/handlers/bin/command-not-found) for openSUSE.

Written in Rust. Uses [libsolv](https://github.com/openSUSE/libsolv) to query the content of all enabled repositories.

## Build

```sh
cargo build
```

## Direct usage

To query not installed programs

```
./target/debug/cnf-rs cmake

The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

To query installed programs in `/usr/sbin`

```
./target/debug/cnf-rs chcpu
Absolute path to 'chcpu' is '/usr/sbin/chcpu', so running it may require superuser privileges (eg. root).
```

To query installed programs in `/usr/bin`

```
./target/debug/cnf-rs vim  
Absolute path to 'vim' is '/usr/bin/vim'. Please check your $PATH variable to see whether it contains the mentioned path
```

##  Integrate with bash

```sh
> source command_not_found_bash
> export COMMAND_NOT_FOUND_BIN=./target/debug/cnf-rs
> cmake
If 'cmake' is not a typo you can use command-not-found to lookup the package that contains it, like this:
    ./target/debug/cnf-rs cmake
> export COMMAND_NOT_FOUND_AUTO=1
> cmake

The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

## TODO

 1. packaging
 2. translation
 3. make default in openSUSE
