# Adapted from https://raw.githubusercontent.com/openSUSE/scout/master/handlers/bin/command_not_found_bash
# under MIT license

command_not_found_handle() {
    export TEXTDOMAINDIR=/usr/share/locale
    export TEXTDOMAIN=cnf
    local cnf_bin=${COMMAND_NOT_FOUND_BIN:-/usr/bin/cnf}

    local cmd state rest
    local -i pid ppid pgrp session tty_nr tpgid

    # do not run when inside Midnight Commander or within a Pipe
    if [[ -n "$MC_SID" || ! -t 1 ]]; then
        eval 'echo $"$1: command not found" >&2'
        return 127
    fi

    # do not run when within a subshell
    # unless $$ == 1, because PID 1 means
    # * process is init - which is unlikely
    # * process was started inside OCI container
    read pid cmd state ppid pgrp session tty_nr tpgid rest  < /proc/self/stat
    if [[ $$ != 1 && $$ -eq $tpgid ]]; then
        eval 'echo $"$1: command not found" >&2'
        return 127
    fi

    # call command-not-found directly
    test -x "${cnf_bin}" && "${cnf_bin}" "$1"

    return 127
}
