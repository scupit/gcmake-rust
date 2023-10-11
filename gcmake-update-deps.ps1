# Until I add equivalent commands for updating valid dependency git repositories and removing invalid
# ones, this script can be used to accomplish both those tasks.

param (
  # Still remove invalid repositories, but don't try to update the valid ones.
  # This is mainly for using this command without an internet connection.
  [switch]
  $cleanOnly,

  [switch]
  $dryRun
)

if ($null -eq (Get-Command "git" -ErrorAction SilentlyContinue)) {
  Write-Error "Could not find an available 'git' command. The git executable must be available in the PATH in order to update gcmake repos."
  exit
}

$repoUpdateScriptBlock = {
  param (
    [string]
    $repoDir
  )

  $dryRun = $using:dryRun
  $cleanOnly = $using:cleanOnly

  if (Test-Path -Path "$repoDir\.git") {
    $depName = Split-Path -Path $(Split-Path -Path $repoDir -Parent) -Leaf
    Set-Location -Path "$repoDir"

    if (git status 2>$null) {
      if (-Not $cleanOnly) {
        # https://stackoverflow.com/questions/18659425/get-git-current-branch-tag-name
        $revisionName = git symbolic-ref --short -q HEAD 
        $revisionType = "branch"
        
        if (-Not $revisionName) {
          $revisionName = git describe --tags --exact-match 2>$null
          $revisionType = "tag"
        }

        if (-Not $revisionName) {
          $revisionName = git rev-parse --short HEAD
          $revisionType = "commit hash"
        }

        $messageBase = "$depName @ $revisionType '$revisionName'"

        if ($dryRun) {
          Write-Host "Would update $messageBase"
        }
        else {
          git checkout $main_branch 2>$null | Out-Null
          (git pull origin "$revisionName" 1>$null 2>$null && Write-Host "Updated $messageBase") || Write-Host "Failed to update $messageBase"
        }
      }
    }
    else {
      if ($dryRun) {
        Write-Host "Would remove invalid repository at '$repoDir'"
      }
      else {
        Remove-Item -Recurse -Force $repoDir && Write-Host "Removed invalid repository at '$repoDir'"
      }
    }
  }
  elseif (-Not (Test-Path -Path "$repoDir/*")) {
    if ($dryRun) {
      Write-Host "Would remove invalid directory (maybe due to failed archive download?) at $repoDir"
    }
    else {
      Remove-Item -Recurse -Force $repoDir && Write-Host "Removed invalid directory (maybe due to failed archive download?) at $repoDir"
    }
  }
}


$gcmakeDepDir = "${env:USERPROFILE}\.gcmake\dep-cache"
$jobList = @()

foreach ($depDir in $(Get-ChildItem -Directory -Path $gcmakeDepDir)) {
  Get-ChildItem -Directory -Path $depDir | ForEach-Object {
    $jobList += Start-ThreadJob -ArgumentList $_ -ScriptBlock $repoUpdateScriptBlock
  }
}

$jobList | Receive-Job -Wait -AutoRemoveJob
