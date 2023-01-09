# Until I add equivalent commands for updating valid dependency git repositories and removing invalid
# ones, this script can be used to accomplish both those tasks.

param (
  # Still remove invalid repositories, but don't try to update the valid ones.
  # This is mainly for using this command without an internet connection.
  [switch]
  $cleanOnly
)

if ($null -eq (Get-Command "git" -ErrorAction SilentlyContinue)) {
  Write-Error "Could not find an available 'git' command. The git executable must be available in the PATH in order to update gcmake repos."
  exit
}

$gcmakeRootDir = "${env:USERPROFILE}\.gcmake\dep-cache"
$invalidRepoDict = [System.Collections.Concurrent.ConcurrentDictionary[string, object]]::new()

$mainJob = $(Get-ChildItem "$gcmakeRootDir") | ForEach-Object -Parallel {
  $repoContainerDir = "$_\git_repo"

  if (Test-Path -Path "$repoContainerDir") {
    $invalidRepoDict = $using:invalidRepoDict
    $cleanOnly = $using:cleanOnly

    $(Get-ChildItem "$repoContainerDir") | ForEach-Object -Parallel {
      $invalidRepoDict = $using:invalidRepoDict
      $cleanOnly = $using:cleanOnly
      $repo_dir = "$_"

      Set-Location -Path "$repo_dir"

      if (-Not (git status 2>$null)) {
        $invalidRepoDict.TryAdd("$repo_dir", "$repo_dir") | Out-Null
      }
      elseif (-Not $cleanOnly) {
        $main_branch = (git remote show origin | findstr 'HEAD branch:') -replace '\s*HEAD branch:\s*', ''
        $main_branch = ($main_branch -split '\n')[0]

        git checkout $main_branch 2>$null | Out-Null
        git pull 1>$null && Write-Output "Updated '$repo_dir'"
      }
    }
  }
} -AsJob

$mainJob | Receive-Job -Wait

foreach ($invalidRepoTuple in $invalidRepoDict.GetEnumerator()) {
  $invalidRepoDir = $invalidRepoTuple.Value
  Remove-Item -Recurse -Force $invalidRepoDir && Write-Output "Removed invalid repository at '$invalidRepoDir'"
}
