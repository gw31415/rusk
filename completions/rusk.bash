complete -F _rusk_completion rusk

_rusk_completion() {
    COMPREPLY=($(compgen -W "$(rusk | awk -F'\t' '{print $1}')" -- "${COMP_WORDS[COMP_CWORD]}"))
}
