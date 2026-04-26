💡 **What:** Replaced synchronous `std::fs::create_dir_all` calls inside `memory-cli/src/config/storage.rs` and `memory-core/src/embeddings/local.rs` with `tokio::fs::create_dir_all().await`. Propagated the async signatures (`ensure_directory_exists` and `get_cache_dir`) appropriately.

🎯 **Why:** To adhere to async/await best practices and prevent blocking the Tokio executor thread during initialization processes. When a synchronous I/O call blocks a worker thread in an asynchronous runtime, it effectively stalls other concurrent tasks until the operation completes, resulting in lower throughput and potential starvation.

📊 **Measured Improvement:**
The optimization focuses on architectural correctness rather than a noticeable absolute speedup. Since directory creation primarily happens once during startup (or lazy initialization), a microbenchmark would only show minimal differences (in the realm of microseconds). Therefore, this change is not evaluated on numerical improvement, but instead guarantees the removal of a blocking sync call inside the Tokio async executor.
