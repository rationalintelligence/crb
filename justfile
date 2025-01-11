version := `toml get Cargo.toml workspace.package.version --raw`
tag := "v" + version

bump:
    cargo set-version --workspace --bump patch

tag:
    git tag {{tag}}
    git push origin {{tag}}

release:
    cargo workspaces publish --allow-branch trunk --all --publish-as-is
