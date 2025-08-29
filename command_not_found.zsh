# Adapted from https://raw.githubusercontent.com/openSUSE/scout/master/handlers/bin/command_not_found_zsh
# under MIT license
#
# define this function with two names
#  - one to be generic command_not_found_handler() and to be hooked
#  - one to be available when command_not_found_handler() is redefined
function command_not_found_handler cnf_handler {
    local cnf_bin=${COMMAND_NOT_FOUND_BIN:-/usr/bin/cnf}
    if [ -x $cnf_bin ]; then
        # take first parameter and remove quotes if there were any so
        # $ 'foo'
        # will search for foo
        $cnf_bin "${(Q)1}"
    fi
}
