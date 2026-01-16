# Storage Backend Mocking Patterns

## MockTursoStorage

```rust
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

pub struct MockTursoStorage {
    episodes: Arc<Mutex<Vec<Episode>>>,
    create_calls: Arc<Mutex<u32>>,
}

impl MockTursoStorage {
    pub fn new() -> Self {
        Self {
            episodes: Arc::new(Mutex::new(Vec::new())),
            create_calls: Arc::new(Mutex::new(0)),
        }
    }

    pub fn expect_create_episode(&self) -> CreateEpisodeMock {
        CreateEpisodeMock {
            storage: self,
            expected_count: None,
            return_value: Ok(()),
        }
    }
}

pub struct CreateEpisodeMock<'a> {
    storage: &'a MockTursoStorage,
    expected_count: Option<u32>,
    return_value: Result<(), StorageError>,
}

impl<'a> CreateEpisodeMock<'a> {
    pub fn times(mut self, n: u32) -> Self {
        self.expected_count = Some(n);
        self
    }

    pub fn returning<F>(self, f: F) -> Self
    where
        F: FnMut(&Episode) -> Result<(), StorageError>,
    {
        self
    }
}

#[async_trait]
impl StorageBackend for MockTursoStorage {
    async fn create_episode(&self, episode: &Episode) -> Result<(), StorageError> {
        let mut calls = self.create_calls.lock().unwrap();
        *calls += 1;

        if let Some(expected) = self.expected_count {
            assert_eq!(*calls, expected,
                "Expected create_episode to be called {} times, but was called {} times",
                expected, *calls);
        }

        self.return_value.clone()?;
        self.episodes.lock().unwrap().push(episode.clone());
        Ok(())
    }

    async fn get_episode(&self, id: &str) -> Result<Option<Episode>, StorageError> {
        Ok(self.episodes.lock().unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned())
    }
}
```

## MockRedbCache

```rust
pub struct MockRedbCache {
    cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MockRedbCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn expect_get(&self) -> GetMock {
        GetMock {
            cache: self,
            expected_key: None,
            return_value: None,
        }
    }
}

pub struct GetMock<'a> {
    cache: &'a MockRedbCache,
    expected_key: Option<String>,
    return_value: Option<Vec<u8>>,
}

impl<'a> GetMock<'a> {
    pub fn returning(self, value: Vec<u8>) -> Self {
        Self {
            return_value: Some(value),
            ..self
        }
    }
}

#[async_trait]
impl CacheBackend for MockRedbCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        if let Some(expected) = &self.expected_key {
            assert_eq!(key, expected,
                "Expected get for key '{}', but got '{}'", expected, key);
        }

        Ok(self.cache.lock().unwrap()
            .get(key)
            .cloned())
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<(), CacheError> {
        self.cache.lock().unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }
}
```

## Usage Example

```rust
#[tokio::test]
async fn test_episode_with_mocks() {
    let mock_turso = MockTursoStorage::new();
    let mock_redb = MockRedbCache::new();

    // Configure expectations
    mock_turso.expect_create_episode()
        .times(1)
        .returning(|_| Ok(()));

    mock_redb.expect_get()
        .returning(vec![]);

    let memory = SelfLearningMemory::with_storage(
        Box::new(mock_turso),
        Box::new(mock_redb),
    ).await?;

    let episode_id = memory.start_episode(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    ).await;

    assert!(!episode_id.is_empty());
}
```
