
# Tab completion for zsh. Load this after compinit.

# Offers the ids of tracked directories (described by their label) and their labels.
__tempit_refs() {
  local -a refs
  local id label
  while IFS=$'\t' read -r id label; do
    if [[ -n $label ]]; then
      refs+=("$id:$label" "$label:#$id")
    else
      refs+=("$id")
    fi
  done < <(command tempit __refs 2>/dev/null)
  _describe -t tempit-refs 'directory' refs
}

_tempit() {
  local context state state_descr line
  typeset -A opt_args
  local -a subcommands=(
    'create:create a directory'
    'new:create a directory'
    'list:list directories'
    'ls:list directories'
    'path:print the path of a directory'
    'remove:delete directories'
    'rm:delete directories'
    'save:move a directory somewhere permanent'
    'clean:delete all directories'
    'init:print the shell integration script'
    'help:print help'
  )
  _arguments -C \
    '(- *)'{-h,--help}'[print help]' \
    '(- *)'{-V,--version}'[print version]' \
    '1: :->command' \
    '*:: :->argument'
  case $state in
    command) _describe -t commands 'tempit command' subcommands ;;
    argument)
      case $line[1] in
        path) _tempit_tempg ;;
        remove | rm) _tempit_temprm ;;
        save) _tempit_tempsave ;;
        clean | clean-all) _tempit_tempclean ;;
        init) _arguments '1:shell:(bash zsh)' ;;
      esac
      ;;
  esac
}

_tempit_tempg() { _arguments '1: :__tempit_refs'; }
_tempit_temprm() { _arguments '*: :__tempit_refs'; }
_tempit_tempsave() { _arguments '1: :__tempit_refs' '2:destination:_files -/'; }
_tempit_tempclean() { _arguments '(-y --yes)'{-y,--yes}'[do not ask for confirmation]'; }

if (( $+functions[compdef] )); then
  compdef _tempit tempit
  compdef _tempit_tempg tempg
  compdef _tempit_temprm temprm
  compdef _tempit_tempsave tempsave
  compdef _tempit_tempclean tempclean
fi
