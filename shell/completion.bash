
# Tab completion for bash.

# Prints the ids and labels of tracked directories, one per line.
__tempit_refs() {
  local __tempit_id __tempit_label
  while IFS=$'\t' read -r __tempit_id __tempit_label; do
    printf '%s\n' "$__tempit_id"
    if [[ -n $__tempit_label ]]; then
      printf '%s\n' "$__tempit_label"
    fi
  done < <(command tempit __refs 2>/dev/null)
}

# Sets COMPREPLY to the given words that start with the word being completed ($1).
__tempit_reply() {
  local __tempit_cur=$1 __tempit_word
  shift
  COMPREPLY=()
  while IFS= read -r __tempit_word; do
    COMPREPLY+=("$__tempit_word")
  done < <(compgen -W "$*" -- "$__tempit_cur")
}

__tempit_complete() {
  local __tempit_cur=${COMP_WORDS[COMP_CWORD]}
  local __tempit_cmd=${COMP_WORDS[0]}
  local __tempit_pos=$COMP_CWORD

  # The temp* functions complete like the subcommand they wrap.
  case $__tempit_cmd in
    tempg) __tempit_cmd=path ;;
    temprm) __tempit_cmd=remove ;;
    tempsave) __tempit_cmd=save ;;
    tempclean) __tempit_cmd=clean ;;
    *)
      if ((__tempit_pos == 1)); then
        __tempit_reply "$__tempit_cur" \
          create new list ls path remove rm save clean init help --help --version
        return
      fi
      __tempit_cmd=${COMP_WORDS[1]}
      __tempit_pos=$((__tempit_pos - 1))
      ;;
  esac

  case $__tempit_cmd in
    path | save)
      # The second argument of `save` falls back to file name completion.
      if ((__tempit_pos == 1)); then
        __tempit_reply "$__tempit_cur" "$(__tempit_refs)"
      fi
      ;;
    remove | rm) __tempit_reply "$__tempit_cur" "$(__tempit_refs)" ;;
    clean | clean-all) __tempit_reply "$__tempit_cur" --yes ;;
    init)
      if ((__tempit_pos == 1)); then
        __tempit_reply "$__tempit_cur" bash zsh
      fi
      ;;
  esac
}

complete -o default -F __tempit_complete tempit tempg temprm tempsave
complete -F __tempit_complete tempclean
