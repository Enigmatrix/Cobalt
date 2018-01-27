# Workflows

## Building
- ```msbuild Cobalt.sln```

## Release New Version
- Add data model migrations (if any) in the [Migrations folder](/Cobalt.Common.Data/Migration/Sqlite/) with a higher version number
- Increase version numbers of all assemblies
- Increase version (only the first 3 numbers matter) in the [Product Definition](Cobalt.Setup/Product.wxs)
- Update version number in [appveyor.yml](/appveyor.yml)
- ```Commit all changes```
- ```git push```
- ```git tag <tag> # e.g. git tag Cobalt-v1.0```
- ```git push origin --tags```
- Edit the drafted release