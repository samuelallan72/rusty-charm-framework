## set up a template that doesn't require charmcraft

- cross-arch builds in rust?
- build.rs that sorts it out for you?

## Remove the test charm once a real charm is implemented for it

## support relations

top level: requires and provides
next level: named endpoint. each endpoint can have multiple applications connected(?)
  does an endpoint need to be unique across requires and provides?

## Support application and unit data bags

## encode all metadata.yaml content in the framework

maybe?

### macro to write the config.yaml, etc. to file at compile time

so the code is the source of truth.
eg.

```
#[proc_macro_attribute]
pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
    todo!()
}
```

### Or maybe the yaml files are the source of truth...

and we have proc macros that build the types and such at compile time?

## dig into all the juju hook tools

and write up docs for what, how, why for all the functions here

## support secrets

```
$ juju help-tool | rg secret
    secret-add               Add a new secret.
    secret-get               Get the content of a secret.
    secret-grant             Grant access to a secret.
    secret-ids               Print secret IDs.
    secret-info-get          Get a secret's metadata info.
    secret-remove            Remove an existing secret.
    secret-revoke            Revoke access to a secret.
    secret-set               Update an existing secret.
```

## support k8s

## support storage management

## support network tools

- unit-get
- network-get

## support payloads

what are these? operator framework does not use the juju `payload-*` hook tools.

## support juju metrics

- add-metric

## support for pebble

## use key/value file input to state-set, etc.

This should be more reliable.
Not sure how it works with key validation - can a key contain an `=` character or not?

## Check for support for types other than string for state/leader-set/etc.
