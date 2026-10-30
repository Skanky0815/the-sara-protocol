use std::{env, net::IpAddr, path::PathBuf};

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub migrations_dir: PathBuf,
}

// Configuration can contain credentials; Debug must never expose them.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("migrations_dir", &self.migrations_dir)
            .field("database_url", &"[REDACTED]")
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_values(|key| env::var(key).ok())
    }

    fn from_values(mut get: impl FnMut(&str) -> Option<String>) -> Result<Self, String> {
        let host = get("HOST").unwrap_or_else(|| "127.0.0.1".into());
        let host = host
            .parse::<IpAddr>()
            .map_err(|_| "HOST must be a valid IP address".to_string())?;
        let port = get("PORT").unwrap_or_else(|| "3000".into());
        let port = port
            .parse::<u16>()
            .map_err(|_| "PORT must be an integer between 1 and 65535".to_string())?;
        if port == 0 {
            return Err("PORT must be between 1 and 65535".into());
        }
        let database_url = get("DATABASE_URL")
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| "DATABASE_URL is required".to_string())?;
        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err("DATABASE_URL must be a PostgreSQL connection URL".into());
        }
        let migrations_dir = get("MIGRATIONS_DIR").unwrap_or_else(|| "./migrations".into());
        if migrations_dir.trim().is_empty() {
            return Err("MIGRATIONS_DIR must not be empty".into());
        }
        Ok(Self {
            host,
            port,
            database_url,
            migrations_dir: migrations_dir.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(values: &[(&str, &str)]) -> Result<Config, String> {
        Config::from_values(|key| {
            values
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).into())
        })
    }

    #[test]
    fn defaults_and_database_url() {
        let cfg = config(&[("DATABASE_URL", "postgres://localhost/test")]).unwrap();
        assert_eq!(cfg.host.to_string(), "127.0.0.1");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.migrations_dir, PathBuf::from("./migrations"));
    }

    #[test]
    fn overrides() {
        let cfg = config(&[
            ("HOST", "0.0.0.0"),
            ("PORT", "8080"),
            ("DATABASE_URL", "postgresql://localhost/test"),
        ])
        .unwrap();
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.host.to_string(), "0.0.0.0");
    }

    #[test]
    fn invalid_port_is_error() {
        assert!(config(&[
            ("PORT", "abc"),
            ("DATABASE_URL", "postgres://localhost/test")
        ])
        .unwrap_err()
        .contains("PORT"));
        assert!(config(&[("PORT", "0"), ("DATABASE_URL", "postgres://localhost/test")]).is_err());
    }

    #[test]
    fn missing_database_url_is_error() {
        assert!(config(&[]).unwrap_err().contains("DATABASE_URL"));
    }
}

#[cfg(test)]
mod migration_config_tests {
    use super::*;

    #[test]
    fn migration_directory_preserves_absolute_and_relative_paths() {
        for path in ["/opt/sara/migrations", "custom/migrations"] {
            let config = Config::from_values(|key| match key {
                "DATABASE_URL" => Some("postgres://localhost/test".into()),
                "MIGRATIONS_DIR" => Some(path.into()),
                _ => None,
            })
            .unwrap();
            assert_eq!(config.migrations_dir, PathBuf::from(path));
        }
    }

    #[test]
    fn blank_migration_directory_is_rejected_without_echoing_value() {
        for value in ["", " \t"] {
            let error = Config::from_values(|key| match key {
                "DATABASE_URL" => Some("postgres://localhost/test".into()),
                "MIGRATIONS_DIR" => Some(value.into()),
                _ => None,
            })
            .unwrap_err();
            assert_eq!(error, "MIGRATIONS_DIR must not be empty");
        }
    }
}
