# tempdir.plugin.zsh

TEMPIT_SCRIPT="${0:A:h}/tempit.py"

_temp_create() {
    local created_dir
    created_dir=$(python ${TEMPIT_SCRIPT} --create $1)
    cd ${created_dir}
}

_temp_list() {
    python ${TEMPIT_SCRIPT} --list
}

_temp_remove() {
    if [ -z $1 ]; then
        python ${TEMPIT_SCRIPT} --help
        return
    fi
    python ${TEMPIT_SCRIPT} --remove $1
}

_temp_go() {
    local target_dir
    target_dir=$(python ${TEMPIT_SCRIPT} --get $1)
    cd ${target_dir}
}

# Aliases
alias tempc="_temp_create"
alias templ="_temp_list"
alias tempr="_temp_remove"
alias tempg="_temp_go"
