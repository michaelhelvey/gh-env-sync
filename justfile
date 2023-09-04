default:
  just run --token $GITHUB_PAT michaelhelvey/gh-env-sync

run *args:
  cargo run -p gh-env-sync -- {{args}}
