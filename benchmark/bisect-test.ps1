# This script is called by `git bisect run`
# Exit 0 = good, exit 1-124 = bad, exit 125 = skip
$msg = git log -1 --format="%s" HEAD 2>$null
if ($msg -match "break|BREAK") { exit 1 }
else { exit 0 }
