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

#### What makes a good error?

- Provides a simplified (Display) and a extended (Debug) version tuned for
multiple audiences.
- Provides the possibility to look at the underlying cause of the error,
if any (source).

When an operation does not produce the desired outcome, we're dealing with an error.

Errors can also be distinguished based on their location:

- Internal (i.e. a function calling another function within our app)
- At the edge (i.e. an API request we failed to fulfill)

|  | Internal | At the edge |
| ------------- | -------------- | -------------- |
| Control Flow | Types, methods, fields |  Status Codes |
| Reporting | Logs/traces | Response Body |

### Tips

- Do not log errors when they are bubbled-up.

### Dealing with Errors in Rust

**Errors serve two main purposes:**

#### Control Flow (i.e. determine what to do next)

- Doing a different action based on a matching system.
- Is scripted: all information required to make a decision should be
accessible to a machine.

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

#### Reporting (e.g. investigate, after the fact, what went wrong)

Error reporting is primarily consumed by **humans**.
