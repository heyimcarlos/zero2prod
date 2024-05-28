# Newsletter API

## Notes

- Before migrating the database, the app needs to have `trusted sources` disabled.
- After deployment to the digital ocean's app platform, migrations have to be applied
manually with the following command:

```bash
DATABASE_URL=<database_url> sqlx migrate run
```

### Trait Objects

#### Box

At the cost of an alloc, hides the original error type from our API. We specify what
type have to be implemented.

```rs
    // using the plain trait doesn't allow for recursiveness (erroring in a nested function)
    trait Error: Debug + Display {
        // `source` allows for a function to grab the source of the error
        fn source(&self) -> Option<&(dyn Error)>
    }
    // I have a pointer to a type, i don't know what type it is, but I know it implements
    // the `Display` and `Debug` traits which is allocated in the heap.
    Box<dyn Error>
```

### Errors

When an operation does not produce the desired outcome, we're dealing with an error.

### Tips

- Do not log errors when they are bubbled-up.

### Dealing with Errors in Rust

#### Control Flows

Doing a different action based on a matching system.

Using an enumeration like:

```rs
// Error enum similar to `Result<V, E>`
enum Fallible<Success, Error>,
where
    Error: Debug + Display
{
    Ok(Success),
    Err {
        error: Error
    }
}

// `TestError` enumeration which allows for finite control flow.
enum TestError {
    RateLimited,
    InvalidInput {},
    GenericError {
        source: Box<dyn Error>
    }
}

fn test() -> Fallible<(), TestError>

match test() {
    Ok(success_val) => { /* */ }
    Err(e) => {
        // [...]
        if let TestError::RateLimited = e.error {
            // [...]
        }
    }
}
```
