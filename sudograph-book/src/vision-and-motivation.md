# Vision and motivation

## Vision

[My](https://twitter.com/lastmjs) goal for Sudograph is for it to become the simplest, most flexible, and in the end most powerful way to develop internet applications. That's the grand vision.

To scope the vision down a bit, more realistically I want Sudograph to become the simplest, most flexible, and in the end most powerful way to develop Internet Computer applications.

Sudograph will achieve this vision by reading your GraphQL schema and generating an amazingly flexible, super simple, infinitely scalable, and extremely secure backend.

That's a lot of hyperbole! But that's also the future I want to build with Sudograph.

Achieving this vision begins with enabling CRUD operations within a single canister, then quickly moves into [migrations](./migrations.md), [authorization](authorization.md), and [multi-canister scaling](./multi-canister-scaling.md).

Sudograph is currently transitioning from alpha into beta, and there's a long journey ahead.

## Motivation

I have been developing with GraphQL since somewhere around 2016. It immediately struck me as a powerful way to deal with the complexities of managing the reading and writing of data for non-trivial internet applications. It has proven to me since that it is extremely versatile, and I have used it for a number of projects with a number of different underlying data sources.

Though GraphQL simplifies development, implementing it is not always simple. It still requires you to write a lot of code to bring your schema to life, in large part because GraphQL does not solve the problem of how data is read and written.

There are a number of libraries that have been developed in the recent past to address this problem. You can think of these as GraphQL generators. They attempt in one way or another to take a GraphQL schema and generate the code required to read and write data.

During my journey to find the perfect GraphQL generator, I went from [Graphcool](https://github.com/Graphcool/graphcool-framework) to [Prisma](https://github.com/prisma/prisma1) to [Graphback](https://github.com/aerogear/graphback) to finally writing a [GraphQL generator from scratch](https://github.com/jillsoffice/graphql-sql-builder). And there are other similar projects out there, like [Hasura](https://github.com/hasura/graphql-engine) and [PostGraphile](https://github.com/graphile/postgraphile).

No project has gotten it right yet, and each library has trade-offs and falls short of the vision of generating an amazingly flexible, super simple, infinitely scalable, and extremely secure backend from a GraphQL schema. It's a very difficult problem.

The Internet Computer may provide a very interesting solution.

The Internet Computer promises flexibility, simplicity, scalability, and security like no other platform before it. Combining the powers of GraphQL with the Internet Computer may be the best chance we have yet to achieve this vision.