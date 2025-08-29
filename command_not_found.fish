function fish_command_not_found
    if test -z "$COMMAND_NOT_FOUND_BIN"
        set -l COMMAND_NOT_FOUND_BIN /usr/bin/cnf
    end
    # The 'env' is there in case your fish is older than v3.0.0
    env $COMMAND_NOT_FOUND_BIN $argv[1]
end