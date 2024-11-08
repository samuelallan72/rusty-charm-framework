## Figure out a good name

## Spin off the test charm into a separate repo or directory

## support relations

## encode all metadata.yaml content in the framework

maybe?

## figure out error handling

remove all unwraps and expects.

## Support application and unit data bags

## macro to write the config.yaml, etc. to file at compile time

so the code is the source of truth.
eg.

```
#[proc_macro_attribute]
pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
    todo!()
}
```
