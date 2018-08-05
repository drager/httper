# HTTPer
[![Build Status](https://travis-ci.org/drager/httper.svg?branch=master)](https://travis-ci.org/drager/httper)

HTTP(S) client built on top of [hyper](https://github.com/hyperium/hyper/).

## Usage

A simple usage example:

```rust
extern crate httper;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

use httper::client::{HttperClient, HttpsClient};
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::<HttpsClient>::new();

    #[derive(Debug, Deserialize, PartialEq)]
    struct Contributor {
        id: u32,
        login: String,
    }

    let result = rt.block_on(
        httper_client
            .get("https://api.github.com/repos/drager/httper/contributors")
            .json::<Vec<Contributor>>(),
    );

    println!("Contributors: {:?}", result);
}
```

## Features and bugs

Please file feature requests and bugs at the [issue tracker][tracker].

[tracker]: https://github.com/drager/httper/issues

## License
Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

