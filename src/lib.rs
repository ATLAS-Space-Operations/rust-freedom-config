#![doc = include_str!("../README.md")]

use std::{fmt::Debug, sync::Arc};

#[cfg(feature = "serde")]
use serde::Deserialize;
use url::Url;

/// A wrapper around T which implements debug and display, without showing the underlying value.
///
/// This is intended to wrap sensitive information, and prevent it from being accidentally logged,
/// or otherwise exposed
#[cfg_attr(feature = "serde", derive(Deserialize), serde(transparent))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Secret<T>(pub T);

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Secret(value)
    }
}

impl<T> Secret<T> {
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Secret").field(&"*****").finish()
    }
}

/// Shared behavior for atlas environments
pub trait AtlasEnv: 'static + AsRef<str> + Debug + Send + Sync + Unpin {
    fn from_str(val: &str) -> Option<Self>
    where
        Self: Sized;

    /// The hostname of the FPS for the given environment
    fn fps_host(&self) -> &str;

    /// The entrypoint for the freedom API for the given environment
    ///
    /// # Note
    ///
    /// Each environment contains the path "/api" as all requests initiate from this point
    fn freedom_entrypoint(&self) -> Url;
}

/// Type state for the test environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Test;

impl AsRef<str> for Test {
    fn as_ref(&self) -> &str {
        "test"
    }
}

impl AtlasEnv for Test {
    fn from_str(val: &str) -> Option<Self>
    where
        Self: Sized,
    {
        val.to_ascii_lowercase().eq("test").then_some(Self)
    }

    fn fps_host(&self) -> &str {
        "fps.test.atlasground.com"
    }

    fn freedom_entrypoint(&self) -> Url {
        Url::parse("https://test-api.atlasground.com/api").unwrap()
    }
}

/// Type state for the production environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Prod;

impl AsRef<str> for Prod {
    fn as_ref(&self) -> &str {
        "prod"
    }
}

impl AtlasEnv for Prod {
    fn from_str(val: &str) -> Option<Self>
    where
        Self: Sized,
    {
        val.to_ascii_lowercase().eq("prod").then_some(Self)
    }

    fn fps_host(&self) -> &str {
        "fps.atlasground.com"
    }

    fn freedom_entrypoint(&self) -> Url {
        Url::parse("https://api.atlasground.com/api").unwrap()
    }
}

/// The configuration object for Freedom.
///
/// Used when creating a Freedom API client
#[derive(Clone, Debug)]
pub struct Config {
    environment: Arc<dyn AtlasEnv>,
    key: String,
    secret: Secret<String>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.environment_str() == other.environment_str()
            && self.key == other.key
            && self.secret == other.secret
    }
}

/// Error enumeration for creating a Freedom Config
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
pub enum Error {
    /// Failed to parse the variable from the environment
    ParseEnvironment,
    /// Missing secret from builder
    MissingSecret,
    /// Missing key from builder
    MissingKey,
    /// Missing environment from builder
    MissingEnvironment,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error {}

/// Builder for the Freedom Config object
#[derive(Default)]
pub struct ConfigBuilder {
    environment: Option<Arc<dyn AtlasEnv>>,
    key: Option<String>,
    secret: Option<Secret<String>>,
}

impl ConfigBuilder {
    /// Construct an empty Config builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempt to load the ATLAS environment from the environment
    pub fn environment_from_env(&mut self) -> Result<&mut Self, Error> {
        let var = std::env::var(Config::ATLAS_ENV_VAR).map_err(|_| Error::ParseEnvironment)?;

        if let Some(env) = Test::from_str(&var) {
            return Ok(self.environment(env));
        }
        if let Some(env) = Prod::from_str(&var) {
            return Ok(self.environment(env));
        }

        Err(Error::ParseEnvironment)
    }

    /// Attempt to load the ATLAS secret from the environment
    pub fn secret_from_env(&mut self) -> Result<&mut Self, Error> {
        let var = std::env::var(Config::ATLAS_SECRET_VAR).map_err(|_| Error::ParseEnvironment)?;

        self.secret(var);
        Ok(self)
    }

    /// Attempt to load the ATLAS key from the environment
    pub fn key_from_env(&mut self) -> Result<&mut Self, Error> {
        let var = std::env::var(Config::ATLAS_KEY_VAR).map_err(|_| Error::ParseEnvironment)?;

        self.key(var);
        Ok(self)
    }

    /// Set the environment
    pub fn environment(&mut self, environment: impl AtlasEnv) -> &mut Self {
        self.environment = Some(Arc::new(environment));
        self
    }

    /// Set the secret
    pub fn secret(&mut self, secret: impl Into<String>) -> &mut Self {
        self.secret = Some(Secret(secret.into()));
        self
    }

    /// Set the key
    pub fn key(&mut self, key: impl Into<String>) -> &mut Self {
        self.key = Some(key.into());
        self
    }

    /// Build the Config from the current builder
    pub fn build(&mut self) -> Result<Config, Error> {
        let Some(environment) = self.environment.take() else {
            return Err(Error::MissingEnvironment);
        };
        let Some(key) = self.key.take() else {
            return Err(Error::MissingKey);
        };
        let Some(secret) = self.secret.take() else {
            return Err(Error::MissingSecret);
        };

        Ok(Config {
            environment,
            key,
            secret,
        })
    }
}

impl Config {
    /// The environment variable name for the atlas environment
    pub const ATLAS_ENV_VAR: &'static str = "ATLAS_ENV";

    /// The environment variable name for the atlas key
    pub const ATLAS_KEY_VAR: &'static str = "ATLAS_KEY";

    /// The environment variable name for the atlas secret
    pub const ATLAS_SECRET_VAR: &'static str = "ATLAS_SECRET";

    /// Construct a new config builder
    ///
    /// # Example
    ///
    /// ```
    /// # use freedom_config::{Config, Test};
    /// let config_result = Config::builder()
    ///     .environment(Test)
    ///     .key("my_key")
    ///     .secret("my_secret")
    ///     .build();
    ///
    /// assert!(config_result.is_ok());
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Build the entire configuration from environment variables
    pub fn from_env() -> Result<Self, Error> {
        Self::builder()
            .environment_from_env()?
            .key_from_env()?
            .secret_from_env()?
            .build()
    }

    /// Construct the Config from the environment, key, and secret
    ///
    /// # Example
    ///
    /// ```
    /// # use freedom_config::{Config, Test};
    /// let config = Config::new(Test, "my_key", "my_secret");
    /// ```
    pub fn new(
        environment: impl AtlasEnv,
        key: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        let environment = Arc::new(environment);

        Self {
            environment,
            key: key.into(),
            secret: Secret(secret.into()),
        }
    }

    /// Set the environment
    ///
    /// # Example
    ///
    /// ```
    /// # let mut config = freedom_config::Config::new(freedom_config::Test, "key", "password");
    /// # use freedom_config::Prod;
    /// config.set_environment(Prod);
    /// assert_eq!(config.environment_str(), "prod");
    /// ```
    pub fn set_environment(&mut self, environment: impl AtlasEnv) {
        self.environment = Arc::new(environment);
    }

    /// Return the trait object representing an ATLAS environment
    pub fn environment(&self) -> &dyn AtlasEnv {
        self.environment.as_ref()
    }

    /// Return the string representation of the environment
    pub fn environment_str(&self) -> &str {
        self.environment.as_ref().as_ref()
    }

    /// Exposes the secret as a string slice.
    ///
    /// # Warning
    ///
    /// Use this with extreme care to avoid accidentally leaking your key
    pub fn expose_secret(&self) -> &str {
        self.secret.expose()
    }

    /// Return the ATLAS key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Set the ATLAS key
    ///
    /// # Example
    ///
    /// ```
    /// # let mut config = freedom_config::Config::new(freedom_config::Test, "key", "password");
    /// config.set_key("top secret");
    /// assert_eq!(config.key(), "top secret");
    /// ```
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    /// Set the value of the ATLAS secret
    ///
    /// # Example
    ///
    /// ```
    /// # let mut config = freedom_config::Config::new(freedom_config::Test, "key", "password");
    /// config.set_secret("top secret");
    /// assert_eq!(config.expose_secret(), "top secret");
    /// ```
    pub fn set_secret(&mut self, secret: impl Into<String>) {
        self.secret = Secret(secret.into());
    }
}
