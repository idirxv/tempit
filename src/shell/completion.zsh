if [[ -n "$__TEMPIT_EXE" ]]; then
  _tempit_complete_candidates() {
    local kind="$1"
    local label="$2"
    local output val desc
    local -a candidates
    output="$(command "$__TEMPIT_EXE" complete "$kind" 2>/dev/null)" || return 0
    [[ -z "$output" ]] && return 0
    while IFS=$'\t' read -r val desc; do
      [[ -n "$val" ]] || continue
      if [[ -n "$desc" ]]; then
        candidates+=("${val//:/\\:}:${desc}")
      else
        candidates+=("${val//:/\\:}")
      fi
    done <<< "$output"
    (( ${#candidates[@]} )) && _describe -t tempit-dirs "$label" candidates
  }

  _tempit_go_complete() {
    if (( CURRENT == 2 )); then
      _tempit_complete_candidates targets 'tracked directory'
    fi
  }

  _tempit_rm_complete() {
    if (( CURRENT == 2 )); then
      _tempit_complete_candidates numbers 'tracked directory number'
    fi
  }

  _tempit_wrapper_complete() {
    case "${words[2]}" in
      go|-g)
        if (( CURRENT == 3 )); then
          _tempit_complete_candidates targets 'tracked directory'
        fi
        ;;
      remove)
        if (( CURRENT == 3 )); then
          _tempit_complete_candidates numbers 'tracked directory number'
        fi
        ;;
      init)
        if (( CURRENT == 3 )); then
          local -a shells=('bash:Bash shell' 'zsh:Zsh shell' 'fish:Fish shell')
          _describe -t shells 'shell' shells
        fi
        ;;
    esac
  }

  _tempit_complete() {
    local context state state_descr line
    typeset -A opt_args
    local -a subcommands=(
      'create:Create a new temporary directory'
      'init:Initialize Tempit in the current shell'
      'list:List all tracked temporary directories'
      'remove:Remove a tracked temporary directory by its number'
      'clean-all:Remove all tracked temporary directories'
      'help:Print help message'
    )

    _arguments -C \
      '(-v --version)'{-v,--version}'[Show version and exit]' \
      '(-h --help)'{-h,--help}'[Print help]' \
      '1:subcommand:->subcmd' \
      '*::arg:->args'

    case "$state" in
      subcmd)
        _describe -t commands 'tempit command' subcommands
        ;;
      args)
        case "${words[1]}" in
          init)
            local -a shells=('bash:Bash shell' 'zsh:Zsh shell' 'fish:Fish shell')
            _arguments -C \
              '(-h --help)'{-h,--help}'[Print help]' \
              '1:shell:->shell'
            if [[ "$state" == shell ]]; then
              _describe -t shells 'shell' shells
            fi
            ;;
          remove)
            _arguments -C \
              '(-h --help)'{-h,--help}'[Print help]' \
              '1:number:->number'
            if [[ "$state" == number ]]; then
              _tempit_complete_candidates numbers 'tracked directory number'
            fi
            ;;
          create)
            _arguments \
              '(-h --help)'{-h,--help}'[Print help]' \
              '1:name:'
            ;;
          list|clean-all)
            _arguments \
              '(-h --help)'{-h,--help}'[Print help]'
            ;;
          help)
            if (( CURRENT == 2 )); then
              _describe -t commands 'tempit command' subcommands
            fi
            ;;
        esac
        ;;
    esac
  }

  if (( $+functions[compdef] )); then
    compdef _tempit_complete tempit
    compdef _tempit_wrapper_complete _tempit
    compdef _tempit_go_complete tempg
    compdef _tempit_rm_complete temprm
  fi
fi
