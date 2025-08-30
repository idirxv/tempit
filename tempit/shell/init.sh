# shellcheck shell=bash
__TEMPIT_EXE="$(command -v tempit)"

if [[ -z "$__TEMPIT_EXE" ]]; then
  echo "tempit: executable not found in PATH" >&2
else
  _tempit() {
    case "$1" in
      create|-c)
        shift
        local __path
        __path="$(command "$__TEMPIT_EXE" --create "$@")" || return $?
        if [[ -n "$__path" && -d "$__path" ]]; then
          cd "$__path" || return $?
        fi
        ;;
      go|-g)
        shift
        local __path
        __path="$(command "$__TEMPIT_EXE" --path "$@")" || return $?
        if [[ -n "$__path" && -d "$__path" ]]; then
          cd "$__path" || return $?
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
    alias templ="_tempit --list"
    alias temprm="_tempit --remove"
    alias tempclean="_tempit --clean-all"

fi
