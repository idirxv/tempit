# shellcheck shell=bash
# tempit shell integration for bash and zsh, printed by `tempit init <shell>`.
# Load it from your shell's rc file:  eval "$(tempit init bash)"   # or zsh

# Runs a tempit subcommand that prints a directory and cd's into it. Any other
# output, such as --help, is printed as is.
__tempit_cd() {
  local __tempit_dir
  __tempit_dir="$(command tempit "$@")" || return
  if [[ -d $__tempit_dir ]]; then
    builtin cd -- "$__tempit_dir" || return
  elif [[ -n $__tempit_dir ]]; then
    printf '%s\n' "$__tempit_dir"
  fi
}

# Runs a tempit subcommand that deletes directories. If it deleted the current
# directory, moves to $HOME rather than staying in a directory that is gone.
__tempit_delete() {
  command tempit "$@"
  local __tempit_status=$?
  [[ -d $PWD ]] || builtin cd -- "$HOME" || return
  return "$__tempit_status"
}

# Create a directory and cd into it: tempc [LABEL]
tempc() { __tempit_cd create "$@"; }

# Go to a directory, by default the most recent: tempg [REF]
tempg() { __tempit_cd path "$@"; }

# List directories: templ
templ() { command tempit list "$@"; }

# Delete directories: temprm REF...
temprm() { __tempit_delete remove "$@"; }

# Delete all directories: tempclean [--yes]
tempclean() { __tempit_delete clean "$@"; }

# Move a directory somewhere permanent: tempsave REF [DEST]
# If the shell is inside it, follows it to its new location.
tempsave() {
  if [[ $# -eq 0 || $1 == -* ]]; then
    command tempit save "$@"
    return
  fi
  local __tempit_src __tempit_dst
  __tempit_src="$(command tempit path -- "$1")" || return
  __tempit_dst="$(command tempit save "$@")" || return
  printf '%s\n' "$__tempit_dst"
  case $PWD/ in
    "$__tempit_src"/*)
      if [[ -d $__tempit_dst ]]; then
        builtin cd -- "$__tempit_dst${PWD#"$__tempit_src"}" || return
      fi
      ;;
  esac
}
