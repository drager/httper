# HTTPer
[![Build Status](https://travis-ci.org/drager/httper.svg?branch=master)](https://travis-ci.org/drager/httper)

A asynchronous HTTP(S) client built on top of [hyper](https://github.com/hyperium/hyper/).

## Why
At the time when I started writting parts of this client
I couldn't find any higher level asynchronous http(s) client. I also tended to
write the same code over and over again for serveral different projects
based on hyper, always wanted to be able to make requests to https addresses
and deserialize the response body into json.

## Usage

A simple usage example:

```rust
extern crate httper;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

use httper::client::HttperClient;
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let httper_client = HttperClient::new();

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

