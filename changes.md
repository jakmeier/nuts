# Version history with summary of changes

## Unreleased
*Crate size: ???*
* New features introduced:
    * Subscription handlers can be registered without an activity. (E.g. `nuts::subscribe(|msg: &MyMessage| {...} )`)

## 0.2.1
*Crate size: 29.4kB*
* New features introduced:
    * More info in logs when running with feature "verbose-debug-log"
    * Private messages can now also have subscription masks, allowing activities to receive private messages when inactive.
* Bugfixes:
    * Fixed an issue where activities ended up using he same data slots. This could happen when activities were added in subscription handlers multiple times with quiescent moments in between.
* Removed features:

## 0.2
*Crate size: 28.3kB*
* New features introduced:
    * Essentially, make it possible to do anything anywhere.
      Previously, some calls were only allowed from outside callbacks. This restriction has been lifted entirely.
      The affected functions are:
        * `new_activity` and all variants of it
        * `subscribe` and all variants of it
        * `on_enter` + `on_leave`
        * `store_to_domain`
    * `LifecycleStatus::Deleted` (which can be used with `set_status`) and `on_delete` which can take ownership of an activity back outside of nuts.
    * `publish_awaiting_response` which returns a future that resolves once the published message has been handled
    * Extended debugging support at runtime (when compiled in debug mode) to give useful information when user code dynamically called by nuts panics.
    * Supporting messages directed at a single activity through `id.private_channel()` or `id.private_domained_channel()` + `nuts::send_to::<Receiver,_>()` or `id.private_message()`
* Removed features
    * `subscribe_owned` and friends (in favour of private messages which do the same without potential runtime panics)

## 0.1.1
*Crate size: 17.7kB*
* Removed dependency (which was only used in examples anyway)
## 0.1
*Crate size: 22.1kB*
* Initial publication on crates.io
