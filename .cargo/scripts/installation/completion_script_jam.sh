_jam_completion() {
    local cur prev words cword
    _init_completion -s || return

    local AllCommands=(
        help h
        query q
        struct tree list ls
        redirect red
        update sync
        commit cmt save sv
        archive
        clone c build
        add new create
        remove rm delete del
        move mv rename
        rollback rb restore
        get g lock
        throw t unlock release
        view v download dl
        param set
    )

    local VFCommands=(
        add new create
        remove rm delete del
        get g lock
        throw t unlock release
        view v download dl
        move mv rename
        rollback rb restore
        query q
    )

    local PArgCommands=(param set)

    if [[ $cword -eq 1 ]]; then
        mapfile -t COMPREPLY < <(compgen -W "${AllCommands[*]}" -- "$cur")
        return
    fi

    local subcmd="${words[1]}"

    if ! [[ " ${AllCommands[*]} " =~ " $subcmd " ]]; then
        mapfile -t COMPREPLY < <(compgen -W "${AllCommands[*]}" -- "$cur")
        return
    fi

    if [[ " ${PArgCommands[*]} " =~ " $subcmd " ]]; then
        local param_dir=".jam/param"
        if [[ -d $param_dir ]]; then
            local params=()
            for f in "$param_dir"/*.txt; do
                [[ -e $f ]] || continue
                params+=("${f##*/}")
                params+=("${f##*/%.txt}")
            done
            mapfile -t COMPREPLY < <(compgen -W "${params[*]}" -- "$cur")
        fi
        return
    fi

    if [[ " ${VFCommands[*]} " =~ " $subcmd " ]]; then
        local prefix fragment
        if [[ $cur == */* ]]; then
            prefix="${cur%/*}/"
            fragment="${cur##*/}"
        else
            prefix=""
            fragment="$cur"
        fi

        if command -v jam &>/dev/null; then
            local suggestions
            suggestions=$(jam query list "$prefix" 2>/dev/null)

            local completions=()
            while IFS= read -r line; do
                [[ -n $line ]] || continue
                if [[ $line == "$fragment"* ]]; then
                    local full_path="$prefix$line"
                    if [[ $full_path == *" "* ]]; then
                        completions+=("\"$full_path\"")
                    else
                        completions+=("$full_path")
                    fi
                fi
            done <<< "$suggestions"

            mapfile -t COMPREPLY < <(compgen -W "${completions[*]}" -- "$cur")
        fi
        return
    fi
}

complete -F _jam_completion jam