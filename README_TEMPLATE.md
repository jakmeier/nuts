# Nuts
@DOC CRATE

## State of Library
With the release of Nuts version 0.2 on [crates.io](https://crates.io/crates/nuts), it has reached an important milestone.
The single-threaded features have all been implemented. Maybe a method here and there needs to be added. But I would not expect to go through major API overhauls again in the existing interface at this point.

There is one big pending feature left, however. This is parallel dispatch, covered in [#2](https://github.com/jakmeier/nuts/issues/2).
Ideally, that would be implemented under the hood. But likely it will make sense to add some more methods to the API.

If and when parallel dispatch get implemented, Nuts probably looks at a stable 1.0 release.

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

## Example: Private Channels
In what I have shown you so far, all messages have been shared reference and it is sent to all listeners that registered to a specific message type.
An alternative is to use private channels. A sender can then decide which listening activity will receive the message.
In that case, the ownership of the message is given to the listener.
@DOC PUBLISH_PRIVATE

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
A simple example using nuts to build a basic clicker game is available in [examples/clicker-game](tree/master/examples/clicker-game). It requires `wasm-pack` installed and `npm`. To run the example, execute `wasm-pack build` and then `cd www; npm install; npm run start`.
This example only shows minimal features of nuts.

Right now, there are no more examples (some had to be removed due to outdated dependencies). Hopefully that will change at some point.

All examples are set up as their own project. (To avoid polluting the libraries dependencies.)
Therefore the standard `cargo run --example` will not work. One has to go to the example's directory and build it from there.
