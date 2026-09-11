# Worktree management. Thin wrappers over worktrunk (`wt`) so the commands stay the same even if
# the underlying tool changes. `wt` does the git plumbing; these recipes add the project
# conventions: branch from a fresh origin/main and the zellij dev layout. `wt` runs through
# `mise exec` so it resolves without shell activation.

wt := "mise exec -- wt"

[doc("Create a worktree (and branch) from a fresh origin/main")]
new branch:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> Fetching origin/main"
    git fetch origin main
    echo "==> Creating worktree for '{{branch}}' from origin/main"
    {{wt}} switch --create "{{branch}}" --base origin/main --no-cd
    echo "==> Ready. 'just wk::list' to see it; 'just wk::zellij {{branch}}' for a dev tab."

[doc("Create a worktree and open it in a zellij tab running the dev layout")]
new_ui branch: (new branch) (zellij branch)

[doc("List all worktrees and their status")]
list:
    @{{wt}} list

[doc("Open an existing worktree in a zellij tab running the dev layout")]
zellij branch:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${ZELLIJ_SESSION_NAME:-}" ]; then
        echo "Error: not inside a zellij session. Start zellij first."
        exit 1
    fi
    worktree_path="$(just wk::_path "{{branch}}")"
    if [ -z "${worktree_path}" ]; then
        echo "Error: no worktree for branch '{{branch}}'. Run 'just wk::new {{branch}}' first."
        exit 1
    fi
    zellij action new-tab \
        --layout "${worktree_path}/.zellij/layout.kdl" \
        --cwd "${worktree_path}" \
        --name "{{branch}}"

[doc("Remove a worktree and delete its branch (pass 'force' to drop an unmerged branch)")]
rm branch=`git branch --show-current` force="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{branch}}" = "main" ]; then
        echo "Error: refusing to remove 'main'."
        exit 1
    fi
    original_dir="${PWD}"
    worktree_path="$(just wk::_path "{{branch}}")"

    echo "==> Removing worktree '{{branch}}'"

    # Run wt from the primary worktree so removing the worktree we are standing in does not pull
    # the cwd out from under the rest of this recipe.
    cd "$(git worktree list --porcelain | awk '/^worktree /{print $2; exit}')"

    # wt removes the worktree and deletes the branch only when it is already merged. 'force'
    # adds --force-delete to drop an unmerged branch too.
    if [ "{{force}}" = "force" ]; then
        {{wt}} remove "{{branch}}" --force --force-delete --foreground
    else
        {{wt}} remove "{{branch}}" --force --foreground
    fi

    # Close the zellij tab only if we removed the worktree we were sitting in.
    if [ -n "${ZELLIJ_SESSION_NAME:-}" ] && [ -n "${worktree_path}" ] \
        && [[ "${original_dir}" == "${worktree_path}"* ]]; then
        zellij action close-tab
    fi

# Print the filesystem path of BRANCH's worktree, empty if there is none. Asks git instead of
# guessing `../<branch>`, so a worktree directory named differently from its branch resolves.
[private]
_path branch:
    @git worktree list --porcelain | awk -v b="refs/heads/{{branch}}" '/^worktree /{p=$2} $0=="branch "b{print p}'
