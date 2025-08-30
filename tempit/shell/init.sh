__TEMPIT_EXE="$(command -v tempit)"

if [[ -z "$__TEMPIT_EXE" ]]; then
  echo "tempit: executable not found in PATH" >&2
else
  _tempit() {
    # Allow `tempit init zsh` to work even after the function is defined.
    if [[ "$1" == "init" ]]; then
      command "$__TEMPIT_EXE" "$@"
      return $?
    fi
    case "$1" in
      create|-c)
        shift
        local __path
        __path="$(command "$__TEMPIT_EXE" --create "$@")" || return $?
        if [[ -n "$__path" && -d "$__path" ]]; then
          cd "$__path"
        fi
        ;;
      go|-g)
        shift
        local __path
        __path="$(command "$__TEMPIT_EXE" --get "$@")" || return $?
        if [[ -n "$__path" && -d "$__path" ]]; then
          cd "$__path"
        fi
        ;;
      *)
        command "$__TEMPIT_EXE" "$@"
        ;;
    esac
  }

    # Aliases
    alias tempc="_tempit create"
    alias tempg="_tempit go"
    alias templ="$__TEMPIT_EXE --list"

fi
