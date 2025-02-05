use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
enum FileError {
    #[error("文件 {path} 读取失败")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("文件不存在: {0}")]
    NotFound(PathBuf),
}

#[cfg(test)]
mod tests_file {
    use std::fs;

    use super::*;
    //测试文件操作

    fn read_file(path: PathBuf) -> Result<String, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound(path));
        }
        fs::read_to_string(&path).map_err(|e| FileError::ReadError { path, source: e })
    }

    #[test]
    fn test_read_example() {
        let file_path = PathBuf::from("example.txt");
        assert!(read_file(file_path).is_err());
    }

    #[test]
    fn test_read_readme() {
        let file_path = PathBuf::from("README.md");
        let result = read_file(file_path);
        assert!(result.is_ok());
        match result {
            Ok(s) => assert!(!s.is_empty(), "文件内容为空"),
            Err(e) => panic!("读取 README.md 文件失败: {:?}", e),
        }
    }

    #[test]
    fn test_read_file_success() -> Result<(), std::io::Error> {
        let test_content = "Hello, World!";
        let temp_file = "test_file.txt";

        // Create a temporary file with test content
        fs::write(temp_file, test_content)?;

        // Test reading the file
        let result = read_file(PathBuf::from(temp_file));

        // Clean up
        fs::remove_file(temp_file)?;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
        Ok(())
    }

    #[test]
    fn test_read_file_not_found() {
        let non_existent_file = PathBuf::from("non_existent_file.txt");
        let result = read_file(non_existent_file.clone());

        match result {
            Err(FileError::NotFound(path)) => assert_eq!(path, non_existent_file),
            _ => panic!("Expected FileError::NotFound"),
        }
    }

    #[test]
    fn test_read_file_permission_error() -> Result<(), std::io::Error> {
        use std::os::unix::fs::PermissionsExt;

        let test_file = "readonly_test.txt";

        // Create a file
        fs::write(test_file, "test content")?;

        // Set permissions to read-only (no write permission)
        fs::set_permissions(test_file, fs::Permissions::from_mode(0o444))?;

        // Try to read the file
        let result = read_file(PathBuf::from(test_file));

        // Clean up (need to make writable again to delete)
        fs::set_permissions(test_file, fs::Permissions::from_mode(0o666))?;
        fs::remove_file(test_file)?;

        match result {
            Err(FileError::ReadError { path, source: _ }) => {
                assert_eq!(path, PathBuf::from(test_file));
                Ok(())
            }
            _ => panic!("Expected FileError::ReadError"),
        }
    }
}

use reqwest;
use std::time::Duration;

#[cfg(test)]
mod tests_web {

    use super::*;
    use tokio;

    #[derive(Debug, Error)]
    enum ApiError {
        #[error("HTTP 请求失败: {0}")]
        RequestFailed(String),

        #[error("TimeOut")]
        TimeOut,
        #[error(transparent)]
        RequestError(#[from] reqwest::Error),
    }

    async fn fetch_data(url: &str) -> Result<String, ApiError> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::RequestFailed(format!("创建 HTTP Client 失败: {}", e)))?;

        let response = client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::TimeOut
                } else {
                    ApiError::RequestFailed(e.to_string())
                }
            })?;
        response.text().await.map_err(ApiError::from)
    }

    #[tokio::test]
    async fn test_api_fetch() {
        let result = fetch_data("https://api.github.com").await;
        assert!(result.is_ok());
        match result {
            Ok(m) => assert!(m.len() > 0),
            Err(_) => panic!("期望返回 InvalidAge 错误"),
        }
    }
}

#[cfg(test)]
mod test_config {
    use serde::Deserialize;
    use serde::Serialize;
    use std::fs;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum ConfigError {
        #[error("配置文件读取失败: {0}")]
        IoError(#[from] std::io::Error),
        #[error("配置解析失败: {0}")]
        ParseError(#[from] toml::de::Error),
        #[error("端口号无效: {0}")]
        InvalidPort(u16),
    }

    #[derive(Deserialize, Serialize)]
    struct Config {
        port: u16,
        host: String,
    }

    fn load_config(path: &str) -> Result<Config, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        if config.port == 0 {
            return Err(ConfigError::InvalidPort(config.port));
        }
        Ok(config)
    }

    #[test]
    fn test_load_config_when_file_not_found() {
        let config = load_config("config.toml");

        assert!(config.is_err());
        match config {
            Err(ConfigError::IoError(e)) => assert!(e.kind() == std::io::ErrorKind::NotFound),
            _ => panic!("期望返回 IoError 错误"),
        }
    }

    #[test]
    fn test_load_config_when_file_is_yaml() -> Result<(), std::io::Error> {
        fs::write(
            "test_config_yaml.yaml",
            r#"
        hst: localhsot
        port: 8080
        "#,
        )?;
        let config = load_config("test_config_yaml.yaml");
        assert!(config.is_err());
        match config {
            Err(e @ ConfigError::ParseError(_)) => {
                eprintln!("完整错误信息: {}", e);
            }
            _ => panic!("期望返回 ParseError 错误"),
        }
        fs::remove_file("test_config_yaml.yaml")?;
        Ok(())
    }

    #[test]
    fn test_invalid_port_value() {
        // 创建一个包含无效端口的 TOML 文件
        fs::write(
            "zero_port_config.toml",
            r#"
            host = "localhost"
            port = 0
        "#,
        )
        .unwrap();

        let result = load_config("zero_port_config.toml");

        assert!(result.is_err());

        match result {
            Err(ConfigError::InvalidPort(port)) => {
                // 验证端口值确实为 0
                assert_eq!(port, 0);
            }
            _ => panic!("Expected InvalidPort error, got a different error type"),
        }

        fs::remove_file("zero_port_config.toml").unwrap();
    }
}

#[cfg(test)]
mod test_db {

    use sqlx::sqlite::SqlitePool;
    use sqlx::Row;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum DbError {
        #[error("数据库连接失败: {0}")]
        ConnectionError(String),
        #[error("查询数据失败: {0}")]
        QueryError(#[from] sqlx::Error),
        #[error("记录不存在： ID= {0}")]
        NotFound(i64),
    }

    // 查询用户信息
    async fn get_user(pool: &SqlitePool, id: i64) -> Result<String, DbError> {
        let result = sqlx::query("SELECT name FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        result
            .map(|row| row.get(0))
            .ok_or_else(|| DbError::NotFound(id))
    }

    #[tokio::test]
    async fn test_db_query() -> Result<(), DbError> {
        // 1. 创建内存中的 SQLite 连接池
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        // 2. 初始化内存数据库表
        sqlx::query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        // 3. 插入测试数据
        sqlx::query("INSERT INTO users (id, name) VALUES (1, 'Alice')")
            .execute(&pool)
            .await?;

        let user = get_user(&pool, 1).await;
        println!("查询结果: {:?}", user);

        assert_eq!(user.unwrap(), "Alice");
        Ok(())
    }
}

#[cfg(test)]
mod test_business {
    use std::error::Error;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum AppError {
        #[error("验证失败")]
        Validation(#[from] ValidationError),
        #[error("业务错误")]
        Business(#[from] BusinessError),
        #[error(transparent)]
        Unknown(#[from] Box<dyn Error + Send + Sync>),
    }

    #[derive(Error, Debug)]
    enum ValidationError {
        #[error("输入无效: {0}")]
        InvalidInput(String),
    }

    #[derive(Error, Debug)]
    enum BusinessError {
        #[error("余额不足: 需要 {required}，当前 {available}")]
        InsufficientFunds { required: f64, available: f64 },
    }

    fn process_payment(amount: f64, balance: f64) -> Result<(), AppError> {
        if amount <= 0.0 {
            return Err(ValidationError::InvalidInput("金额必须大于0".to_string()).into());
        }
        if amount > balance {
            return Err(BusinessError::InsufficientFunds {
                required: amount,
                available: balance,
            }
            .into());
        }
        Ok(())
    }

    #[test]
    fn test_business() {
        let result = process_payment(100.0, 50.0);
        println!("支付结果: {:?}", result);
        let result = process_payment(-10.0, 100.0);
        println!("支付结果: {:?}", result);
    }
}
