if ! command -v git 2>/dev/null > /dev/null; then
	echo "Unable to find 'git' on the system. Make sure git is installed."
	exit
fi

update_or_fix_single_repo() {
	repo_dir="$1"
	clean_only_mode=$2
	dry_run=$3
  dep_name=$(basename $(dirname $repo_dir))

  if [[ -d "$repo_dir/.git" ]]; then
    cd "$repo_dir" || exit

    if git status 2>/dev/null > /dev/null; then
      if [[ ! "$clean_only_mode" ]]; then
        revision_name=$(git symbolic-ref --short -q HEAD)
        revision_type='branch'

        if [[ -z "$revision_name" ]]
          revision_name=$(git describe --tags --exact-match 2> /dev/null)
          revision_type='tag'
        fi

        if [[ -z "$revision_name" ]]
          revision_name=$(git rev-parse --short HEAD)
          revision_type='commit hash'
        fi

        message_base="$dep_name @ $revision_type '$revision_name'"

        if [[ $dry_run ]]; then
          echo "Would update $message_base"
        else
          git checkout $revision_name 2>&1 > /dev/null
          (git pull origin $revision_name 2>&1 > /dev/null && echo "Updated $message_base") || echo "Failed to update $message_base"
        fi
      fi
    else
      if [[ $dry_run ]]; then
        echo "Would remove invalid repository at '$repo_dir'"
      else
        rm -r "$repo_dir" && echo "Removed invalid repository at $repo_dir"
      fi
    fi
  fi
}

do_repo_updates() {
	clean_only_mode=$1
	dry_run=$2
	gcmake_dep_dir="$HOME/.gcmake/dep-cache"

	for dep_dir in $gcmake_dep_dir/*; do
    for repo_dir in $dep_dir; do
      if [ -d $repo_container_dir ]; then
        update_or_fix_single_repo "$repo_dir" $clean_only_mode $dry_run &
      fi
    done
	done

	wait $(jobs -p)
}

while getopts "cd" flags; do
	case "$flags" in
		c)
			clean_only=1
			;;
		d)
			dry_run=1
			;;
		*)
			echo "Invalid flag '$flags' given. Specify either -c (clean only mode) or no flags at all."
			;;
	esac
done

do_repo_updates $clean_only $dry_run
