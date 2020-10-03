# Nuts
@DOC CRATE

Nuts has reached a minimal viable product (MVP) with version 0.1 published on [crates.io](https://crates.io/crates/nuts).
But there are more features planned for the future and breaking API changes are very much possible.

## Activities
@DOC ACTIVITY

## Publish
@DOC PUBLISH

## Subscribe 
Activities can subscribe to messages, based on the Rust type identifier of the message. Closures or function pointers can be used to create a subscription for a specific type of messages.


Nuts uses `core::any::TypeId` internally to compare the types. Subscriptions are called when the type of a published message matches the message type of the subscription.

There are several different methods for creating new subscriptions. The simplest of them is simply called `subscribe(...)` and it can be used like this:

@DOC SUBSCRIBE_EXAMPLE

## Example: Basic Activity with Publish + Subscribe
@DOC NEW_ACTIVITY

## Activity Lifecycle
@DOC ACTIVITY_LIFECYCLE

## Domains

@DOC DOMAIN

### Creating Domains
Nuts creates domains implicitly in the background. The user can simply provide an enum or struct that implements the `DomainEnumeration` trait. This trait requires only the `fn id(&self) -> usize` function, which maps every object to a number representing the domain.

Typically, domains are defined by an enum and the `DomainEnumeration` trait is derived using using [`domain_enum!`](macro.domain_enum.html). 

@DOC DOMAIN_MACRO_EXAMPLE

### Using Domains
The function [`nuts::store_to_domain`](fn.store_to_domain.html) allows to initialize data in a domain. Afterwards, the data is available in subscription functions of the activities.

@DOC DOMAIN_STORE

If activities are associated with a domain, they must be registered using the [`nuts::new_domained_activity`](fn.new_domained_activity.html).
This will allow to subscribe with closures that have access to domain state.
[`subscribe_domained`](struct.ActivityId.html#method.subscribe_domained) is used to add those subscriptions.
[`subscribe`](struct.ActivityId.html#method.subscribe) can still be used for subscriptions that do not access the domain.

#### Example of Activity with Domain
@DOC NEW_ACTIVITY_WITH_DOMAIN

## Advanced: Understanding the Execution Order
@DOC PUBLISH_ADVANCED

## Full Demo Examples
A simple example using nuts to build a basic clicker game is available in [examples/clicker-game](tree/master/examples/clicker-game). It requires `wasm-pack` installed to install the package and then `npm run start` in the `www` folder can be run to start a server running the game.
This example only shows minimal features of nuts.

There is another example available in [examples/webstd-example](tree/master/examples/webstd-example).
It shows how Nuts can be combined with just [stdweb](https://github.com/koute/stdweb) to build a web application.
It uses multiple activities with domains and lifecycle status changes.
This example uses [cargo-web](https://github.com/koute/cargo-web) and can be compiled **without** `wasm-pack` or `npm` installed.

All examples are set up as their own project. (To avoid polluting the libraries dependencies.)
Therefore the standard `cargo run --example` will not work. One has to go to the example's directory and build it from there.
