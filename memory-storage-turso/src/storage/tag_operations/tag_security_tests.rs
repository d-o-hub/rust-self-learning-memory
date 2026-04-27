use super::{MAX_TAGS_PER_QUERY, find_episodes_by_tags_and, find_episodes_by_tags_or};
use crate::Error;
use libsql::Builder;

async fn setup_db() -> libsql::Connection {
    let db = Builder::new_local(":memory:").build().await.unwrap();
    db.connect().unwrap()
}

#[tokio::test]
async fn test_max_tags_per_query_or() {
    let conn = setup_db().await;
    let mut tags = Vec::new();
    for i in 0..=MAX_TAGS_PER_QUERY {
        tags.push(format!("tag{}", i));
    }

    let result = find_episodes_by_tags_or(&conn, &tags, None).await;
    assert!(result.is_err());
    match result {
        Err(Error::Storage(msg)) => assert!(msg.contains("Too many tags in query")),
        _ => panic!("Expected Storage error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_max_tags_per_query_and() {
    let conn = setup_db().await;
    let mut tags = Vec::new();
    for i in 0..=MAX_TAGS_PER_QUERY {
        tags.push(format!("tag{}", i));
    }

    let result = find_episodes_by_tags_and(&conn, &tags, None).await;
    assert!(result.is_err());
    match result {
        Err(Error::Storage(msg)) => assert!(msg.contains("Too many tags in query")),
        _ => panic!("Expected Storage error, got {:?}", result),
    }
}
