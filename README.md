# HTTPer

HTTP(S) client built on top of [hyper](https://github.com/hyperium/hyper/).

## Usage

A simple usage example:

```rust
extern crate httper;

#[macro_use]
extern crate serde_derive;

use httper::client::{HttperClient, HttpsClient};

fn main() {
    #[derive(Debug, Deserialize)]
    struct Data {
        name: String,
    }

    let httper_client = HttperClient::<HttpsClient>::new();

    let data = Data {
        name: "Optimus Prime".to_string(),
    };

    httper_client.get_json::<Data>("https://testing.local");
}
```

## Features and bugs

Please file feature requests and bugs at the [issue tracker][tracker].

[tracker]: https://github.com/drager/httper/issues

## License
Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

