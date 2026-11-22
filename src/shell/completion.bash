# shellcheck shell=bash
if [[ -n "$__TEMPIT_EXE" ]]; then
  _tempit_reply_words() {
    local cur="$1"
    shift
    local word
    for word in "$@"; do
      if [[ "$word" == "$cur"* ]]; then
        COMPREPLY+=("$word")
      fi
    done
  }

  _tempit_complete_candidates() {
    local kind="$1"
    local cur="$2"
    local output val
    output="$(command "$__TEMPIT_EXE" complete "$kind" 2>/dev/null)" || return 0
    [[ -z "$output" ]] && return 0
    while IFS=$'\t' read -r val _; do
      if [[ -n "$val" && "$val" == "$cur"* ]]; then
        COMPREPLY+=("$val")
      fi
    done <<< "$output"
  }

  _tempit_go_complete() {
    COMPREPLY=()
    if [[ "$COMP_CWORD" -eq 1 ]]; then
      _tempit_complete_candidates targets "${COMP_WORDS[COMP_CWORD]}"
    fi
  }

  _tempit_rm_complete() {
    COMPREPLY=()
    if [[ "$COMP_CWORD" -eq 1 ]]; then
      _tempit_complete_candidates numbers "${COMP_WORDS[COMP_CWORD]}"
    fi
  }

  _tempit_complete() {
    COMPREPLY=()
    local cur="${COMP_WORDS[COMP_CWORD]}"
    if [[ "$COMP_CWORD" -eq 1 ]]; then
      _tempit_reply_words "$cur" create init list remove clean-all help -v --version -h --help
      return 0
    fi

    local subcmd="${COMP_WORDS[1]}"
    case "$subcmd" in
      init)
        if [[ "$COMP_CWORD" -eq 2 ]]; then
          _tempit_reply_words "$cur" bash zsh fish -h --help
        fi
        ;;
      remove)
        if [[ "$COMP_CWORD" -eq 2 ]]; then
          if [[ "$cur" == -* ]]; then
            _tempit_reply_words "$cur" -h --help
          else
            _tempit_complete_candidates numbers "$cur"
          fi
        fi
        ;;
      create|list|clean-all)
        if [[ "$COMP_CWORD" -eq 2 && "$cur" == -* ]]; then
          _tempit_reply_words "$cur" -h --help
        fi
        ;;
      help)
        if [[ "$COMP_CWORD" -eq 2 ]]; then
          _tempit_reply_words "$cur" create init list remove clean-all help
        fi
        ;;
    esac
  }

  complete -F _tempit_complete tempit
  complete -F _tempit_go_complete tempg
  complete -F _tempit_rm_complete temprm
fi
