if ! command -v git 2>/dev/null > /dev/null; then
	echo "Unable to find 'git' on the system. Make sure git is installed."
	exit
fi

update_or_fix_single_repo() {
	repo_dir="$1"
	clean_only_mode="$2"

	cd "$repo_dir" || exit

	if git status 2>/dev/null > /dev/null; then
		if [ ! "$clean_only_mode" ]; then
			main_branch=$(git remote show origin | grep 'HEAD branch:' | head | sed -e 's/ *HEAD branch: *//g')
			git checkout "$main_branch" 2>/dev/null > /dev/null
			git pull >/dev/null && echo "Updated $repo_dir"
		fi
	else
		rm -r "$repo_dir" && echo "Removed invalid repository at $repo_dir"
	fi
}

do_repo_updates() {
	clean_only_mode=$1
	gcmake_dep_dir="$HOME/.gcmake/dep-cache"

	for dep_dir in "$gcmake_dep_dir"/*; do
		repo_container_dir="$dep_dir/git_repo"

		if [ -d "$repo_container_dir" ]; then
			for repo_dir in "$repo_container_dir"/*; do
				update_or_fix_single_repo "$repo_dir" "$clean_only_mode" &
			done
		fi
	done

	wait $(jobs -p)
}

while getopts ":c" flags; do
	case "$flags" in
		c)
			clean_only=1
			;;
		*)
			echo "Invalid flag '$flags' given. Specify either -c (clean only mode) or no flags at all."
			;;
	esac
done

do_repo_updates "$clean_only"

