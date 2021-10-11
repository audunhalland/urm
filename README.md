# Urm
Turn your database into a queryable graph. But stop for a sec - not in the conventional boring way you've now seen so many times. Urm works a bit differently.

Conventional database-to-graph libraries/services present an interface where the database schema is more or less automatically mirrored into
a GraphQL schema, where the hooks for injecting business logic are more awkward parts of the API.

The Urm philosphy emerged from a set of beliefs:
1. Code is code and configuration is configuration. Don't blur the line.
2. Boilerplate code can and should be generated from code, instead of configuration.
3. A configurable SaaS is most often not what you really want.
4. One-size-fits-all is not a thing that exists. Flexibility for the developer should be maximized (therefore you want a library).
5. You do not want to expose your database schema to the whole wide internet without any abstraction.

This leads us to the Urm design.

## Design
Urm is a library written in Rust. As a user of the library, you'll write Rust.

Urm utilizes Rust's excellent macro system to generate your boilerplate code.
