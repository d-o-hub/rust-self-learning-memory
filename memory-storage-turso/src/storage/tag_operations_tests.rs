use super::*;

async fn setup_test_db() -> Connection {
    let db = libsql::Builder::new_local(":memory:")
        .build()
        .await
        .unwrap();
    let conn = db.connect().unwrap();

    // Create tables
    conn.execute(
        r#"
            CREATE TABLE episodes (
                episode_id TEXT PRIMARY KEY NOT NULL
            )
            "#,
        (),
    )
    .await
    .unwrap();

    conn.execute(
        r#"
            CREATE TABLE episode_tags (
                episode_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (episode_id, tag)
            )
            "#,
        (),
    )
    .await
    .unwrap();

    conn.execute(
        r#"
            CREATE TABLE tag_metadata (
                tag TEXT PRIMARY KEY NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 0,
                first_used INTEGER NOT NULL,
                last_used INTEGER NOT NULL
            )
            "#,
        (),
    )
    .await
    .unwrap();

    conn
}

#[tokio::test]
async fn test_save_and_get_tags() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();

    // Insert episode
    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();

    let tags = vec!["bug-fix".to_string(), "critical".to_string()];

    save_episode_tags(&conn, &episode_id, &tags).await.unwrap();

    let retrieved_tags = get_episode_tags(&conn, &episode_id).await.unwrap();
    assert_eq!(retrieved_tags.len(), 2);
    assert!(retrieved_tags.contains(&"bug-fix".to_string()));
    assert!(retrieved_tags.contains(&"critical".to_string()));
}

#[tokio::test]
async fn test_delete_tags() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();

    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();

    let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
    save_episode_tags(&conn, &episode_id, &tags).await.unwrap();

    delete_episode_tags(&conn, &episode_id, &["tag2".to_string()])
        .await
        .unwrap();

    let remaining_tags = get_episode_tags(&conn, &episode_id).await.unwrap();
    assert_eq!(remaining_tags.len(), 2);
    assert!(!remaining_tags.contains(&"tag2".to_string()));
}

#[tokio::test]
async fn test_find_by_tags_or() {
    let conn = setup_test_db().await;
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    for ep in [&ep1, &ep2] {
        conn.execute(
            "INSERT INTO episodes (episode_id) VALUES (?)",
            [ep.to_string()],
        )
        .await
        .unwrap();
    }

    save_episode_tags(&conn, &ep1, &["tag1".to_string(), "tag2".to_string()])
        .await
        .unwrap();
    save_episode_tags(&conn, &ep2, &["tag2".to_string(), "tag3".to_string()])
        .await
        .unwrap();

    let results = find_episodes_by_tags_or(&conn, &["tag1".to_string()], None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&ep1));

    let results = find_episodes_by_tags_or(&conn, &["tag2".to_string()], None)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_find_by_tags_and() {
    let conn = setup_test_db().await;
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    for ep in [&ep1, &ep2] {
        conn.execute(
            "INSERT INTO episodes (episode_id) VALUES (?)",
            [ep.to_string()],
        )
        .await
        .unwrap();
    }

    save_episode_tags(&conn, &ep1, &["tag1".to_string(), "tag2".to_string()])
        .await
        .unwrap();
    save_episode_tags(&conn, &ep2, &["tag2".to_string(), "tag3".to_string()])
        .await
        .unwrap();

    let results = find_episodes_by_tags_and(&conn, &["tag1".to_string(), "tag2".to_string()], None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&ep1));

    let results = find_episodes_by_tags_and(&conn, &["tag2".to_string()], None)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_tag_statistics() {
    let conn = setup_test_db().await;
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    for ep in [&ep1, &ep2] {
        conn.execute(
            "INSERT INTO episodes (episode_id) VALUES (?)",
            [ep.to_string()],
        )
        .await
        .unwrap();
    }

    save_episode_tags(&conn, &ep1, &["tag1".to_string()])
        .await
        .unwrap();
    save_episode_tags(&conn, &ep2, &["tag1".to_string(), "tag2".to_string()])
        .await
        .unwrap();

    let stats = get_tag_statistics(&conn).await.unwrap();
    assert_eq!(stats.len(), 2);
    assert_eq!(stats.get("tag1").unwrap().usage_count, 2);
    assert_eq!(stats.get("tag2").unwrap().usage_count, 1);
}
