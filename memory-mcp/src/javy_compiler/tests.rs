//! Tests for Javy compiler

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_javy_compiler_creation() {
        let config = JavyConfig::default();
        let compiler = JavyCompiler::new(config);
        assert!(compiler.is_ok());
    }

    #[tokio::test]
    async fn test_js_syntax_validation() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        assert!(compiler.validate_js_syntax("const x = 1;").is_ok());
        assert!(compiler.validate_js_syntax("const x = {;").is_err());
    }

    #[tokio::test]
    async fn test_metrics_initialization() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        let m = compiler.get_metrics().await;
        assert_eq!(m.total_compilations, 0);
        assert_eq!(m.successful_compilations, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        let k1 = compiler.generate_cache_key("const x = 1;");
        let k2 = compiler.generate_cache_key("const x = 1;");
        let k3 = compiler.generate_cache_key("const x = 2;");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}
