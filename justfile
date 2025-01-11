version := `toml get Cargo.toml workspace.package.version --raw`
tag := "v{{version}}"

bump:
    cargo set-version --workspace --bump patch

release:
    git tag {{tag}}
    git push {{tag}}
    cargo workspaces publish --allow-branch trunk --all --publish-as-is
