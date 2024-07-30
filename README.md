# Freedom Config

Utilities for creating an ATLAS Freedom configuration

## Usage

The `freedom-config` API provides an easy to use builder type for constructing
the configuration, which can be invoked with the following:

```rust
use freedom_config::{Config, Test};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = Config::builder()
	    .environment(Test)
		.key("my_key")
		.secret("my_secret")
		.build()?;
		
	println!("{:?}", config);
	
	Ok(())
}
```

The builder can also be used to source these items from the environment:

```rust
use freedom_config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	std::env::set_var(Config::ATLAS_ENV_VAR, "Test");
	std::env::set_var(Config::ATLAS_KEY_VAR, "my_key");
	std::env::set_var(Config::ATLAS_SECRET_VAR, "my_secret");

	let config = Config::builder()
	    .environment_from_env()?
		.key_from_env()?
		.secret_from_env()?
		.build()?;
		
	println!("{:?}", config);
	
	Ok(())
}
```

Since this is fairly common, there is also a shorthand for constructing the
config entirely from environment variables

```rust
use freedom_config::Config;

fn main() {
	let config = Config::from_env();
}
```
