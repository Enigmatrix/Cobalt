# Workflows

## Building
- ```msbuild Cobalt.sln```

## Release New Version
- Add data model migrations (if any) in the [Migrations folder](/Cobalt.Common.Data/Migration/Sqlite/) with a higher version number
- Increase version (only the first 3 numbers matter) in the [Product Definition](Cobalt.Setup/Product.wxs)
- ```git tag <tag> # e.g. git tag 1.0```
- ```git push origin --tags```
- Edit the drafted release
- Delete the previous 'release' (e.g. delete 1.0 and keep Cobalt-v1.0)