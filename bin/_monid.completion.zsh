#compdef monid
_monid_completion() {
  _arguments \
    '(-h --help)'{-h,--help}'[Afficher l aide]' \
    '--completion[Generer la completion shell]:shell:(zsh)'
}

compdef _monid_completion monid
