# Initialize Git repo and push
# Run after Git is installed: .\init-and-push.ps1 [<remote-url>]

Set-Location $PSScriptRoot

git init
git add .
git status
git commit -m "Initial commit: MultiverseX shipping d-apps master repository"

if ($args.Count -gt 0) {
    $remote = $args[0]
    git remote add origin $remote
    git branch -M main
    git push -u origin main
} else {
    Write-Host ""
    Write-Host "To push, create a repo on GitHub and run:"
    Write-Host "  git remote add origin https://github.com/YOUR_ORG/mvx-dapps.git"
    Write-Host "  git branch -M main"
    Write-Host "  git push -u origin main"
}
