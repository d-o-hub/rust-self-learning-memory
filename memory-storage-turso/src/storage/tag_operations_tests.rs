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

// ---------------------------------------------------------------------------
// Transaction rollback on every failure path (issue #1088)
// ---------------------------------------------------------------------------

/// Deterministic failure injection: abort any statement that deletes a row whose tag
/// is `sentinel`. SQLite's default `ABORT` conflict resolution undoes the failing
/// statement but leaves the surrounding transaction open, which is exactly the state a
/// missing `ROLLBACK` would leak into the pooled connection.
async fn inject_delete_failure(conn: &Connection, table: &str, sentinel: &str) -> String {
    let trigger = format!("inject_{sentinel}_delete");
    let sql = format!(
        "CREATE TRIGGER {trigger} BEFORE DELETE ON {table} \
         FOR EACH ROW WHEN OLD.tag = '{sentinel}' \
         BEGIN SELECT RAISE(ABORT, 'injected delete failure'); END"
    );
    conn.execute(&sql, ()).await.unwrap();
    trigger
}

/// Same injection for the insert path (`NEW.tag` instead of `OLD.tag`).
async fn inject_insert_failure(conn: &Connection, table: &str, sentinel: &str) -> String {
    let trigger = format!("inject_{sentinel}_insert");
    let sql = format!(
        "CREATE TRIGGER {trigger} BEFORE INSERT ON {table} \
         FOR EACH ROW WHEN NEW.tag = '{sentinel}' \
         BEGIN SELECT RAISE(ABORT, 'injected insert failure'); END"
    );
    conn.execute(&sql, ()).await.unwrap();
    trigger
}

/// Remove an injected failure so the follow-up happy-path assertion can run.
async fn remove_injection(conn: &Connection, trigger: &str) {
    conn.execute(&format!("DROP TRIGGER {trigger}"), ())
        .await
        .unwrap();
}

/// Fail the operation after the existing tag rows are gone but before commit, then
/// assert the previous tag set survived and the connection is reusable.
async fn assert_replacement_rolls_back(
    conn: &Connection,
    episode_id: &Uuid,
    failing: &[String],
    trigger: Option<&str>,
) {
    let before = get_episode_tags(conn, episode_id).await.unwrap();
    let stats_before = get_tag_statistics(conn).await.unwrap();

    let err = save_episode_tags(conn, episode_id, failing)
        .await
        .expect_err("injected failure must abort the tag replacement");
    let err_msg = err.to_string();

    // (a) the previous tag set is still intact - the delete was rolled back too.
    assert_eq!(
        get_episode_tags(conn, episode_id).await.unwrap(),
        before,
        "failed replacement must not drop existing tags ({err_msg})"
    );

    // (b) no transaction is left open on this connection: a fresh one can be started
    // and committed. Without a ROLLBACK SQLite rejects the BEGIN with
    // "cannot start a transaction within a transaction".
    conn.execute("BEGIN TRANSACTION", ())
        .await
        .unwrap_or_else(|e| panic!("connection left in an open transaction after: {err_msg}: {e}"));
    conn.execute("ROLLBACK", ()).await.unwrap();

    if let Some(trigger) = trigger {
        remove_injection(conn, trigger).await;
    }

    // (c) a successful replacement commits tags and usage counts exactly once.
    save_episode_tags(conn, episode_id, &["delta".to_string()])
        .await
        .expect("tag replacement must work after a rolled-back attempt");
    assert_eq!(
        get_episode_tags(conn, episode_id).await.unwrap(),
        vec!["delta".to_string()]
    );

    let stats = get_tag_statistics(conn).await.unwrap();
    for (tag, before) in &stats_before {
        assert_eq!(
            stats.get(tag).map(|s| s.usage_count),
            Some(before.usage_count),
            "usage_count of '{tag}' must be untouched by the rolled-back attempt"
        );
    }
    assert_eq!(stats.get("delta").unwrap().usage_count, 1);
}

#[tokio::test]
async fn test_save_episode_tags_rolls_back_when_delete_of_existing_tags_fails() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();
    save_episode_tags(
        &conn,
        &episode_id,
        &["keepme".to_string(), "beta".to_string()],
    )
    .await
    .unwrap();

    let trigger = inject_delete_failure(&conn, "episode_tags", "keepme").await;

    assert_replacement_rolls_back(
        &conn,
        &episode_id,
        &["gamma".to_string()],
        Some(trigger.as_str()),
    )
    .await;
    // The rolled-back attempt must not have committed its new tag either.
    let stats = get_tag_statistics(&conn).await.unwrap();
    assert!(
        !stats.contains_key("gamma"),
        "aborted insert must not persist metadata"
    );
}

#[tokio::test]
async fn test_save_episode_tags_rolls_back_when_insert_fails_mid_transaction() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();
    save_episode_tags(
        &conn,
        &episode_id,
        &["alpha".to_string(), "beta".to_string()],
    )
    .await
    .unwrap();

    // No trigger needed: `PRIMARY KEY (episode_id, tag)` rejects the second insert of
    // the duplicated tag, which happens after the existing rows were deleted.
    assert_replacement_rolls_back(
        &conn,
        &episode_id,
        &["gamma".to_string(), "gamma".to_string()],
        None,
    )
    .await;

    let stats = get_tag_statistics(&conn).await.unwrap();
    assert!(
        !stats.contains_key("gamma"),
        "usage_count increment of the aborted attempt must be rolled back"
    );
}

#[tokio::test]
async fn test_save_episode_tags_rolls_back_when_metadata_update_fails() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();
    save_episode_tags(
        &conn,
        &episode_id,
        &["alpha".to_string(), "beta".to_string()],
    )
    .await
    .unwrap();

    let trigger = inject_insert_failure(&conn, "tag_metadata", "boom").await;

    // "ok" inserts cleanly first, so the failure lands after half the work is done.
    assert_replacement_rolls_back(
        &conn,
        &episode_id,
        &["ok".to_string(), "boom".to_string()],
        Some(trigger.as_str()),
    )
    .await;

    let stats = get_tag_statistics(&conn).await.unwrap();
    assert!(
        !stats.contains_key("ok"),
        "metadata of the successfully inserted tag must be rolled back with the rest"
    );
}

#[tokio::test]
async fn test_delete_episode_tags_rolls_back_and_leaves_connection_usable() {
    let conn = setup_test_db().await;
    let episode_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO episodes (episode_id) VALUES (?)",
        [episode_id.to_string()],
    )
    .await
    .unwrap();
    save_episode_tags(
        &conn,
        &episode_id,
        &["alpha".to_string(), "boom".to_string(), "gamma".to_string()],
    )
    .await
    .unwrap();

    let trigger = inject_delete_failure(&conn, "episode_tags", "boom").await;

    let err = delete_episode_tags(
        &conn,
        &episode_id,
        &["alpha".to_string(), "boom".to_string()],
    )
    .await
    .expect_err("injected failure must abort the tag delete");
    let err_msg = err.to_string();

    // Nothing was deleted: the whole statement is undone by the rollback.
    assert_eq!(
        get_episode_tags(&conn, &episode_id).await.unwrap(),
        vec!["alpha".to_string(), "boom".to_string(), "gamma".to_string()],
        "failed delete must not remove any tag ({err_msg})"
    );

    // The connection is not sitting in an open transaction.
    conn.execute("BEGIN TRANSACTION", ())
        .await
        .unwrap_or_else(|e| panic!("connection left in an open transaction: {err_msg}: {e}"));
    conn.execute("ROLLBACK", ()).await.unwrap();

    // And the same delete works once the injected failure is gone.
    remove_injection(&conn, &trigger).await;
    delete_episode_tags(
        &conn,
        &episode_id,
        &["alpha".to_string(), "boom".to_string()],
    )
    .await
    .unwrap();
    assert_eq!(
        get_episode_tags(&conn, &episode_id).await.unwrap(),
        vec!["gamma".to_string()]
    );
}
