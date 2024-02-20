# **Command not found handler for openSUSE**

1.  A replacement for a [python based `command-not-found` handler](https://github.com/openSUSE/scout/blob/master/handlers/bin/command-not-found) for openSUSE.
2.  Written in Rust, so has as few runtime dependencies as is possible.
3.  Uses [`libsolv`](https://github.com/openSUSE/libsolv) under the hood, so is 100% compatible with `zypper`.

## **Differences**

|                                    | **`cnf`**         | **`scout(cnf)`**              |
|------------------------------------|-------------------|-------------------------------|
| **Uses**                           | `libsolv`         | `libsov`                      |
| **Written in**                     | Rust              | shell, two packages in Python |
| **Detect enabled/disabled repos?** | Yes               | Yes                           |
| **Tries to refresh repos**         | No                | Yes                           |
| **`bash` integration**             | Yes               | Yes                           |
| **`zsh` integration**              | Yes               | Yes                           |
| **Disable integration**            | Uninstall package | Magic variable                |
| **Localization**                   | Yes (UTF-8 only)  | Yes                           |

## **Build**

```.sh
cargo build
```

## **Direct usage**

To query not installed programs

```.sh
./target/debug/cnf cmake
```
```.log
The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

To query installed programs in `/usr/sbin`:

```.sh
./target/debug/cnf chcpu
```
```.log
Absolute path to 'chcpu' is '/usr/sbin/chcpu', so running it may require superuser privileges (eg. root).
```

To query installed programs in `/usr/bin`:

```.sh
./target/debug/cnf vim
```
```.log
Absolute path to 'vim' is '/usr/bin/vim'. Please check your $PATH variable to see whether it contains the mentioned path
```

##  **Integrate with `bash`**

```.sh
source command_not_found_bash
export COMMAND_NOT_FOUND_BIN=./target/debug/cnf
cmake
```
```.log
The program 'cmake' can be found in following packages:
  * cmake-full [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]
  * cmake-mini [ path: /usr/bin/cmake, repository: zypp (repo-oss) ]

Try installing with:
    sudo zypper install <selected_package>
```

## **Integration tests**

Integration tests runs inside docker image tagged `local/cnf-ci`. It is built as a part of Github Action and can be built locally as `cd test; docker build -t local/cnf-ci:latest .`

The testing itself is wrapped in [bats](https://github.com/bats-core/bats-core) and in order to make it run, one needs to initialize the git submodules (`git submodule init`). Then tests can be executed using a following command

```.sh
$ ./test/bats/bin/bats ./test/
```
```.log
test.bats
 ✓ root: installed /usr/bin/rpm
 ✓ root: installed /usr/sbin/sysctl
 ✓ root: not installed single package
 ✓ root: not installed more packages
 ✓ root: bash handler: not installed more packages
 ✓ nonroot: not installed more packages

6 tests, 0 failures
```

Every test can be executed on a command line. The `root.sh` wrapper mounts the binary to `/usr/bin/cnf` and mounts the `libsolv.so.1` if running on ubuntu-amd64 or if shared library is in `test/libsolv.so.1`. This is done in order to solve the packaging difference of a libsolv between openSUSE and Ubuntu.

```.sh
./root.sh /usr/bin/cnf rpm
```
```.log
Absolute path to 'rpm' is '/usr/bin/rpm'. Please check your $PATH variable to see whether it contains the mentioned path
```

## **PowerShell users**

As `cnf` does not integrate with PowerShell by default, please read the issue comment https://github.com/openSUSE/cnf/issues/8#issuecomment-1854638267 to learn how to configure your system properly:

<blockQuote>

Here's an example `command_not_found.ps1` file that somewhat emulates what `command_not_found.bash` does:

```.PS1
$env:TEXTDOMAINDIR = '/usr/share/locale'
$env:TEXTDOMAIN = 'cnf'

$ExecutionContext.InvokeCommand.CommandNotFoundAction = {

  param($commandAsSubmitted, $commandLookupEventArgs)

  # Ignore the following invocations:
  # *   via a 'get-*' prefix stemming from PowerShell's default-verb logic,
  # *   via a repeated lookup in the _current_ dir. (./…) - PowerShell itself will provide feedback on that.
  # *   calls from scripts.
  if ($commandAsSubmitted -like 'Get-*' -or $commandAsSubmitted -like '.[/\]*' -or (Get-PSCallStack)[1].ScriptName) { return }

  # Determine the executable path.
  $cnf_bin = $env:COMMAND_NOT_FOUND_BIN ?? '/usr/bin/cnf'

  # Invoke and output the results directly to the terminal (host).
  & $cnf_bin $commandAsSubmitted *>&1 | Out-Host

  # Note: 
  # *   PowerShell's not-found error is seemingly invariably printed afterwards;
  #     the following does NOT prevent that:
  #       $commandLookupEventArgs.StopSearch = $true
  # *   Supposedly, returning a [System.Management.Automation.CommandInfo] instance
  #     - such as output by Get-Command - can provide the command to execute, but that 
  #     doesn't seem to work.

}
```

    *   This script can be invoked directly (doesn't need sourcing); e.g.. `./command_not_found.ps1`

    *   A call to this script (or the code it contains) is best placed in the user's [`$PROFILE`](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_Profiles) file.

    *   As in the Bash case, the mechanism supports only _one_ custom command-not-found handler.
      
        *   However, a plug-in system is being worked on that supports multiple third-party handlers, called _feedback providers_ - see [this blog post](https://devblogs.microsoft.com/powershell/what-are-feedback-providers/).
        *   Someone could write a custom feedback provider that incorporates `cnf`, similar to the one in the [`command-not-found` provider](https://github.com/PowerShell/command-not-found); the latter currently relies solely on the presence of `/usr/lib/command-not-found`, as preinstalled on Ubuntu distros.

</blockQuote>
