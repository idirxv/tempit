# tempit.plugin.zsh - Manage temporary directories with persistent tracking

# Find the script path more reliably
TEMPIT_SCRIPT="${0:A:h}/tempit.py"

# Use python3 if available, fallback to python
if command -v python3 >/dev/null 2>&1; then
    TEMPIT_PY_CMD="python3"
else
    TEMPIT_PY_CMD="python"
fi

_temp_create() {
    local created_dir
    created_dir=$(${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --create $1)
    if [[ -d "$created_dir" ]]; then
        cd "${created_dir}"
        echo "Created and moved to: ${created_dir}"
    else
        echo "Error: Failed to create temporary directory"
        return 1
    fi
}

_temp_list() {
    ${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --list
}

_temp_remove() {
    if [ -z "$1" ]; then
        echo "Error: Directory number required"
        echo "Usage: tempr NUMBER"
        return 1
    fi
    ${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --remove $1
}

_temp_go() {
    if [ -z "$1" ]; then
        echo "Error: Directory number required"
        echo "Usage: tempg NUMBER"
        return 1
    fi

    local target_dir
    target_dir=$(${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --get $1)

    if [[ -d "$target_dir" ]]; then
        cd "${target_dir}"
        echo "Moved to: ${target_dir}"
    else
        echo "Error: Invalid directory number or directory doesn't exist"
        return 1
    fi
}

_temp_search() {
    if [ -z "$1" ]; then
        echo "Error: Search term required"
        echo "Usage: temps SEARCH_TERM"
        return 1
    fi

    ${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --search "$1"
}

_temp_clean_all() {
    echo "Warning: This will remove ALL tracked temporary directories."
    echo "Are you sure? (y/N)"
    read -r confirm

    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        ${TEMPIT_PY_CMD} ${TEMPIT_SCRIPT} --clean-all
    else
        echo "Operation cancelled."
    fi
}

_temp_help() {
    echo "Tempit - Temporary Directory Manager"
    echo ""
    echo "Commands:"
    echo "  tempc [prefix] - Create a new temporary directory (optional prefix)"
    echo "  templ         - List all tracked temporary directories"
    echo "  tempg NUMBER  - Go to the specified temporary directory by number"
    echo "  tempr NUMBER  - Remove the specified temporary directory by number"
    echo "  temps TERM    - Search for directories containing the specified term"
    echo "  tempclean     - Remove all tracked temporary directories"
    echo "  temph         - Show this help message"
}



# Aliases
alias tempc="_temp_create"
alias templ="_temp_list"
alias tempr="_temp_remove"
alias tempg="_temp_go"
alias temps="_temp_search"
alias tempclean="_temp_clean_all"
alias temph="_temp_help"
