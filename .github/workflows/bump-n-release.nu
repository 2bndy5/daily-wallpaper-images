# This script automates the release process for all of the packages in this repository.
# In order, this script does the following:
#
# 1. Bump version number in Cargo.toml manifest.
#
#    This step requires `cargo-edit` (specifically `set-version` feature) installed.
#
# 2. Pushes the changes from steps 1 to remote


# Run an external command and output its elapsed time.
#
# Not useful if you need to capture the command's output.
export def --wrapped run-cmd [...cmd: string] {
    let app = if (
        ($cmd | first) == "git"
        or ($cmd | first) == "gh"
    ) {
        ($cmd | first 2) | str join " "
    } else if ($cmd | first) == 'uvx' {
        $cmd | skip 1 | first
    } else {
        ($cmd | first)
    }
    print $"(ansi blue)\nRunning(ansi reset) ($cmd | str join ' ')"
    let elapsed = timeit {|| ^($cmd | first) ...($cmd | skip 1)}
    print $"(ansi magenta)($app) took ($elapsed)(ansi reset)"
}

# Bump the version.
#
# This function also updates known occurrences of the old version spec to
# the new version spec in various places (like README.md and action.yml).
export def bump-version [
    component: string # The version component (major, minor, patch) to bump.
    --dry-run, # Prevent this function from making changes to disk/files.
] {
    let spec = open pubspec.yaml
    let old = (
        $spec
        | get version
        | str trim
    )
    let old_spec = (
        $old
        | parse "{major}.{minor}.{patch}"
        | first
        | update major { $in | into int }
        | update minor { $in | into int }
        | update patch { $in | into int }
    )
    let new = if ($component == "major") {
        $old_spec | update major { $in + 1 } | update minor 0 | update patch 0
    } else if ($component == "minor") {
        $old_spec | update minor { $in + 1 } | update patch 0
    } else if ($component == "patch") {
        $old_spec | update patch { $in + 1 }
    } else {
        error make {msg: $"Invalid version component: ($component)"}
    }
    let new = $"($new.major).($new.minor).($new.patch)"
    if (not $dry_run) {
        $spec | update version $"($new)" | save --force pubspec.yaml
    }
    print $"bumped ($old) to ($new)"
    $new
}

export const RELEASE_NOTES = $nu.temp-dir | path join "ReleaseNotes.md"
const CHANGELOG = "CHANGELOG.md"

# Is the the default branch currently checked out?
export def is-on-main [] {
    let branch = (
        ^git branch
        | lines
        | where {$in | str starts-with '*'}
        | first
        | str trim --left --char '*'
        | str trim
    ) == "main"
    $branch
}

# The main function of this script.
#
# The acceptable `component` values are what `cargo set-version --bump` accepts:
#
# - manor
# - minor
# - patch
export def main [
    component?: string, # If not provided, `git-cliff` will guess the next version based on unreleased git history.
] {
    let is_main = is-on-main
    let ver = if not $is_main {
        bump-version $component --dry-run
    } else {
        bump-version $component
    }
    let tag = $"v($ver)"
    let is_ci = $env | get --optional CI | into bool --relaxed
    if not $is_main {
        let prompt = "Not checked out on default branch!"
        if ($is_ci) {
            print $"::error::($prompt)"
        } else {
            print $"(ansi yellow)($prompt)(ansi reset)"
        }
        exit 1
    }
    if ($is_ci) {
        run-cmd git config --global user.name $"($env.GITHUB_ACTOR)"
        run-cmd git config --global user.email $"($env.GITHUB_ACTOR_ID)+($env.GITHUB_ACTOR)@users.noreply.github.com"
    }
    run-cmd git add --all
    run-cmd git commit -m $"build: bump version to ($tag)"
    run-cmd git push

    print $"Deploying ($tag)"
    run git tag $tag
    run git push origin $tag
}
