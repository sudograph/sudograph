# Sudograph settings

There will be many settings that Sudograph will allow the developer to customize. Currently however, none of these settings are possible to change. But here is how it might eventually look to change settings:

```graphql
type SudographSettings {
    exportGeneratedGraphQLQueryFunction: false
    exportGenerateGraphQLMutationFunction: true
    exportGeneratedInitFunction: true
    exportGeneratedPostUpgradeFunction: false
}
```