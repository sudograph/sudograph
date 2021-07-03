# Schema

The schema is where you define all of the data types of your application, including relations between types. It is also where you will eventually define many other settings, possibly including authentication, authorization, subnet, and Sudograph-specific settings.

An example schema might look like this:

```graphql
type User {
    id: ID!
    username: String!
    blogPosts: [BlogPost!]! @relation(name: "User:blogPosts::BlogPost:author")
}

type BlogPost {
    id: ID!
    publishedAt: Date
    title: String!
    author: User! @relation(name: "User:blogPosts::BlogPost:author")
}
```

We have told Sudograph that we have two object types, `User` and `BlogPost`. We've described the fields of each type, using some included scalar types such as `ID`, `Date`, and `String`. We have also described one relation between our two types, a one-to-many relationship from `User` to `BlogPost` on the fields `User:blogPosts` and `BlogPost:author`.

The schema is an incredibly powerful yet simple tool for defining the complex data types of your application. Get to know the possibilities of your schema:

* [Scalars](./schema-scalars.md)
* [Enums](./schema-enums.md)
* [Objects](./schema-objects.md)
* [Relations](./schema-relations.md)
* [Custom scalars](./schema-custom-scalars.md)
* [Custom resolvers](./schema-custom-resolvers.md)
* [Custom directives](./schema-custom-directives.md)
* [Sudograph directives](./schema-sudograph-directives.md)
* [Sudograph settings](./schema-sudograph-settings.md)