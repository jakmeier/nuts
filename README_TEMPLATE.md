# Nuts
@DOC CRATE

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
There is currently one example available in `examples/webstd.rs`.
It shows how Nuts can be combined with [stdweb](https://github.com/koute/stdweb) to build a web application.
It uses multiple activities with domains and lifecycle status changes. 

## WIP
This library is still work-in-progress.
Hopefully, a first publication on cates.io is coming soon.

TODO: Fix README links
TODO: Fix example test (broken because it tries to compile without stdweb)
TODO: Write a blog post about the motivation behind Nuts
TODO: Cleanup documentation, add more examples where necessary