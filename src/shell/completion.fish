if test -n "$__TEMPIT_EXE"
    complete -c tempit -e
    complete -c tempg -e
    complete -c temprm -e

    complete -c tempit -n __fish_use_subcommand -s v -l version -d "Show version and exit"
    complete -c tempit -n __fish_use_subcommand -s h -l help -d "Print help"
    complete -c tempit -f -n __fish_use_subcommand -a create -d "Create a new temporary directory"
    complete -c tempit -f -n __fish_use_subcommand -a init -d "Initialize Tempit in the current shell"
    complete -c tempit -f -n __fish_use_subcommand -a list -d "List all tracked temporary directories"
    complete -c tempit -f -n __fish_use_subcommand -a remove -d "Remove a tracked temporary directory by its number"
    complete -c tempit -f -n __fish_use_subcommand -a clean-all -d "Remove all tracked temporary directories"
    complete -c tempit -f -n __fish_use_subcommand -a help -d "Print help"

    complete -c tempit -f -n "__fish_seen_subcommand_from init; and test (count (commandline -opc)) -eq 2" -a "bash zsh fish"
    complete -c tempit -f -n "__fish_seen_subcommand_from remove; and test (count (commandline -opc)) -eq 2" -a '(command "$__TEMPIT_EXE" complete numbers 2>/dev/null)'
    complete -c tempit -f -n "__fish_seen_subcommand_from create init list remove clean-all" -s h -l help -d "Print help"

    complete -c tempg -f -n "test (count (commandline -opc)) -eq 1" -a '(command "$__TEMPIT_EXE" complete targets 2>/dev/null)'
    complete -c temprm -f -n "test (count (commandline -opc)) -eq 1" -a '(command "$__TEMPIT_EXE" complete numbers 2>/dev/null)'
end
