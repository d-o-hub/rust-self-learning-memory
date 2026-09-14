window.BENCHMARK_DATA = {
  "lastUpdate": 1789408758660,
  "repoUrl": "https://github.com/d-o-hub/rust-self-learning-memory",
  "entries": {
    "Rust Benchmarks": [
      {
        "commit": {
          "author": {
            "name": "d.o.",
            "username": "d-o-hub",
            "email": "242170972+d-o-hub@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf838662f60548f2ef5fbf1bd03ea56f9710e5e",
          "message": "Merge pull request #997 from d-o-hub/release-prep-v0.1.40\n\nchore(release): prepare v0.1.40 changelog and version docs",
          "timestamp": "2026-09-06T18:27:55Z",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/eaf838662f60548f2ef5fbf1bd03ea56f9710e5e"
        },
        "date": 1788749605028,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 47918,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18328,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 179035,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31344,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156200,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 41547430,
            "range": "± 1222897",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 376736356,
            "range": "± 4347581",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 189318011,
            "range": "± 4328508",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 374035389,
            "range": "± 6675380",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 78812395,
            "range": "± 1485505",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 23496289,
            "range": "± 690795",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 180,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 276,
            "range": "± 963",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2274441,
            "range": "± 43255",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 405,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3649,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 192,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 231,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 281,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 207,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3751,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 41341,
            "range": "± 1274",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 438717,
            "range": "± 1810",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2236882,
            "range": "± 33234",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 178,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12885,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1231,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 387,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 424,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 478,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 404,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 547,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 739,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2810,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74218,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 316,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7496,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1423,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 36770,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161895,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41841,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 659362,
            "range": "± 4719",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10692,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9927259,
            "range": "± 919944",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9575734,
            "range": "± 325983",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 148363,
            "range": "± 1863",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 80308,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4295314132,
            "range": "± 22037095",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4441425657,
            "range": "± 36551857",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4599617208,
            "range": "± 34617288",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4884905052,
            "range": "± 18320170",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4301484531,
            "range": "± 39270874",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4423920036,
            "range": "± 16393317",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4608849603,
            "range": "± 42763592",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4646531003,
            "range": "± 14207231",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4273705813,
            "range": "± 22186781",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4351517961,
            "range": "± 20898135",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4452488594,
            "range": "± 17808963",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7884555473,
            "range": "± 46298356",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4470557824,
            "range": "± 20841591",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5193570583,
            "range": "± 52392835",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6048439625,
            "range": "± 35847271",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34495,
            "range": "± 1104",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 888612,
            "range": "± 18677",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 18085,
            "range": "± 2103",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 51534,
            "range": "± 5501",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 109942,
            "range": "± 22078",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 159485,
            "range": "± 21010",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 345,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2791,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1422,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7464,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 73735,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 36777,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23524,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 45586,
            "range": "± 9211",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 52649,
            "range": "± 10452",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2189,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1023,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 979,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 914,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 658,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 531,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5352321,
            "range": "± 235968",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2723605,
            "range": "± 212246",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18682,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 923,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28173,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12869,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 57641,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7630,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5209,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14128,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3897,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35418,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349090,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 189322,
            "range": "± 8090",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14715,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1080,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 33182,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3049,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 366,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50226,
            "range": "± 539",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8155437b2dd1cae7dfcfc11f774ac44117003ffb",
          "message": "Merge pull request #998 from d-o-hub/chore/bump-0.1.41\n\nchore(release): bump workspace to 0.1.41 after shipping v0.1.40",
          "timestamp": "2026-09-07T09:12:50+02:00",
          "tree_id": "2be630dd85b9f4bf0c5fe2f3b3d8dfe0dbf9da94",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/8155437b2dd1cae7dfcfc11f774ac44117003ffb"
        },
        "date": 1788768271545,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48122,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18275,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 179093,
            "range": "± 617",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31507,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 155611,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 41936520,
            "range": "± 1612808",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 376263249,
            "range": "± 4476511",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 189292225,
            "range": "± 2831112",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 374600455,
            "range": "± 6699925",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 76164210,
            "range": "± 836005",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 24584289,
            "range": "± 529388",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 182,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 277,
            "range": "± 960",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2294919,
            "range": "± 52554",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 465,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3710,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 196,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 235,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 285,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 211,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3737,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 41189,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 432645,
            "range": "± 3529",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2241241,
            "range": "± 14287",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 186,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12880,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1224,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 391,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 435,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 487,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 412,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 551,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 731,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2785,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74633,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7494,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1435,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37164,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 160905,
            "range": "± 809",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41677,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 668857,
            "range": "± 12785",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10478,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9662028,
            "range": "± 194070",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9787610,
            "range": "± 271329",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 152724,
            "range": "± 2642",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 82915,
            "range": "± 2213",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4306124606,
            "range": "± 39373707",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4431132693,
            "range": "± 18833911",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4605466125,
            "range": "± 44707454",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4885074854,
            "range": "± 28373334",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4299170204,
            "range": "± 28491257",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4434366478,
            "range": "± 15108028",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4604612655,
            "range": "± 21622677",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4656142234,
            "range": "± 19205578",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4274495297,
            "range": "± 20506591",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4347721414,
            "range": "± 20639053",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4446142394,
            "range": "± 18124999",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7852294758,
            "range": "± 22109229",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4479610505,
            "range": "± 35767115",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5175453731,
            "range": "± 22277454",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6035507587,
            "range": "± 38147908",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 33828,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 827887,
            "range": "± 4220",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 18289,
            "range": "± 3590",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 50290,
            "range": "± 6935",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 102282,
            "range": "± 13618",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 157831,
            "range": "± 18137",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 335,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2787,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1422,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7557,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74883,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37281,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23993,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 54792,
            "range": "± 24928",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 50024,
            "range": "± 4024",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2239,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 972,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 971,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 919,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 662,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 529,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5620965,
            "range": "± 409770",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2701345,
            "range": "± 100850",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18575,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 903,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28141,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12896,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 57921,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7479,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5049,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13812,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3851,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35178,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 344016,
            "range": "± 1061",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 186667,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14599,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1099,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3869,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 241,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50692,
            "range": "± 396",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e2084861dc1e50bb322c2376171030f94fc983d4",
          "message": "ci(deps): bump the actions-all group across 1 directory with 3 updates (#1001)\n\nBumps the actions-all group with 3 updates in the / directory: [taiki-e/install-action](https://github.com/taiki-e/install-action), [actions/deploy-pages](https://github.com/actions/deploy-pages) and [reviewdog/action-actionlint](https://github.com/reviewdog/action-actionlint).\n\n\nUpdates `taiki-e/install-action` from 2.85.13 to 2.87.5\n- [Release notes](https://github.com/taiki-e/install-action/releases)\n- [Changelog](https://github.com/taiki-e/install-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/taiki-e/install-action/compare/82cd3e7658a6f96c86c0234aeeda1748937cb0a1...5bf6ce016fd2e72eefc647cbca1e4213f65955b8)\n\nUpdates `actions/deploy-pages` from 5.0.0 to 5.0.1\n- [Release notes](https://github.com/actions/deploy-pages/releases)\n- [Commits](https://github.com/actions/deploy-pages/compare/cd2ce8fcbc39b97be8ca5fce6e763baed58fa128...368f82528645a54fb793d4d04e342629a3f51346)\n\nUpdates `reviewdog/action-actionlint` from 1.73.2 to 1.73.4\n- [Release notes](https://github.com/reviewdog/action-actionlint/releases)\n- [Commits](https://github.com/reviewdog/action-actionlint/compare/dbe5299849118fd6f099ba563d263d770955a64a...d290e336d5a743810aef4404f757dc862276d2ae)\n\n---\nupdated-dependencies:\n- dependency-name: taiki-e/install-action\n  dependency-version: 2.87.5\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: actions-all\n- dependency-name: actions/deploy-pages\n  dependency-version: 5.0.1\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: actions-all\n- dependency-name: reviewdog/action-actionlint\n  dependency-version: 1.73.4\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: actions-all\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-07T17:41:10+02:00",
          "tree_id": "90fc6a5a75eca1a2b43864a0c0ac75d84b71e586",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/e2084861dc1e50bb322c2376171030f94fc983d4"
        },
        "date": 1788799555031,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 50006,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18322,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 190706,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 32574,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156832,
            "range": "± 2645",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 35815515,
            "range": "± 549626",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 369369285,
            "range": "± 6424030",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 182193367,
            "range": "± 4223550",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 370012046,
            "range": "± 6326279",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 70209811,
            "range": "± 1508912",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 19314068,
            "range": "± 545507",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 197,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 390,
            "range": "± 1981",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2121839,
            "range": "± 18897",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 451,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3869,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 201,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 244,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 301,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 224,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3727,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 42313,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 445732,
            "range": "± 1989",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2271391,
            "range": "± 67916",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 189,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 14499,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1351,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 389,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 427,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 483,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 405,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 596,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 797,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 62,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2905,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 81625,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 318,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 8277,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1430,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 41053,
            "range": "± 1167",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161533,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41278,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 659111,
            "range": "± 8699",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10485,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 7726496,
            "range": "± 325676",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 7716897,
            "range": "± 377862",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 132396,
            "range": "± 1957",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 73545,
            "range": "± 670",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4263004137,
            "range": "± 14819999",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4391851861,
            "range": "± 4309601",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4555399634,
            "range": "± 13778457",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4839290224,
            "range": "± 33542060",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4266674049,
            "range": "± 17047650",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4405428467,
            "range": "± 17105102",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4556411090,
            "range": "± 26279988",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4643571398,
            "range": "± 22656892",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4260812262,
            "range": "± 16268591",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4335274953,
            "range": "± 17794607",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4444086901,
            "range": "± 10346142",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7854754290,
            "range": "± 79135780",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4451638138,
            "range": "± 27891617",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5156556748,
            "range": "± 12725375",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6018313878,
            "range": "± 24308520",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 36059,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 897099,
            "range": "± 3503",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 17663,
            "range": "± 1439",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 50591,
            "range": "± 2412",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 103105,
            "range": "± 5919",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 161780,
            "range": "± 16936",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 313,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2815,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1435,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 8328,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 82044,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 41020,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 24141,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 45348,
            "range": "± 10434",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 56544,
            "range": "± 12355",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2020,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1042,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1043,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 990,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 668,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 523,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 4261250,
            "range": "± 167531",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2114431,
            "range": "± 72441",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 19176,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 988,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29646,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13479,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 61011,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7907,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5248,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14679,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 4053,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 37606,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 370194,
            "range": "± 21423",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 16567,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1147,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3697,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 343,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 48983,
            "range": "± 589",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d91a01b83cb5e6911826fdbb1a93e364942eaf0",
          "message": "chore(deps): bump the rust-patch-minor group across 1 directory with 10 updates (#1002)\n\nBumps the rust-patch-minor group with 10 updates in the / directory:\n\n| Package | From | To |\n| --- | --- | --- |\n| [async-trait](https://github.com/dtolnay/async-trait) | `0.1.91` | `0.1.92` |\n| [thiserror](https://github.com/dtolnay/thiserror) | `2.0.19` | `2.0.20` |\n| [redb](https://github.com/cberner/redb) | `4.1.0` | `4.2.0` |\n| [uuid](https://github.com/uuid-rs/uuid) | `1.24.0` | `1.26.0` |\n| [futures](https://github.com/rust-lang/futures-rs) | `0.3.33` | `0.3.34` |\n| [lru](https://github.com/jeromefroe/lru-rs) | `0.18.2` | `0.18.4` |\n| [toml](https://github.com/toml-rs/toml) | `1.1.4+spec-1.1.0` | `1.1.5+spec-1.1.0` |\n| [tokenizers](https://github.com/huggingface/tokenizers) | `0.23.1` | `0.23.2` |\n| [flate2](https://github.com/rust-lang/flate2-rs) | `1.1.9` | `1.1.10` |\n| [which](https://github.com/harryfei/which-rs) | `8.0.5` | `8.0.6` |\n\n\n\nUpdates `async-trait` from 0.1.91 to 0.1.92\n- [Release notes](https://github.com/dtolnay/async-trait/releases)\n- [Commits](https://github.com/dtolnay/async-trait/compare/0.1.91...0.1.92)\n\nUpdates `thiserror` from 2.0.19 to 2.0.20\n- [Release notes](https://github.com/dtolnay/thiserror/releases)\n- [Commits](https://github.com/dtolnay/thiserror/compare/2.0.19...2.0.20)\n\nUpdates `redb` from 4.1.0 to 4.2.0\n- [Release notes](https://github.com/cberner/redb/releases)\n- [Changelog](https://github.com/cberner/redb/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/cberner/redb/compare/v4.1.0...v4.2.0)\n\nUpdates `uuid` from 1.24.0 to 1.26.0\n- [Release notes](https://github.com/uuid-rs/uuid/releases)\n- [Commits](https://github.com/uuid-rs/uuid/compare/v1.24.0...v1.26.0)\n\nUpdates `futures` from 0.3.33 to 0.3.34\n- [Release notes](https://github.com/rust-lang/futures-rs/releases)\n- [Changelog](https://github.com/rust-lang/futures-rs/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/rust-lang/futures-rs/compare/0.3.33...0.3.34)\n\nUpdates `lru` from 0.18.2 to 0.18.4\n- [Changelog](https://github.com/jeromefroe/lru-rs/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/jeromefroe/lru-rs/compare/0.18.2...0.18.4)\n\nUpdates `toml` from 1.1.4+spec-1.1.0 to 1.1.5+spec-1.1.0\n- [Commits](https://github.com/toml-rs/toml/compare/toml-v1.1.4...toml-v1.1.5)\n\nUpdates `tokenizers` from 0.23.1 to 0.23.2\n- [Release notes](https://github.com/huggingface/tokenizers/releases)\n- [Changelog](https://github.com/huggingface/tokenizers/blob/main/RELEASE.md)\n- [Commits](https://github.com/huggingface/tokenizers/compare/v0.23.1...v0.23.2)\n\nUpdates `flate2` from 1.1.9 to 1.1.10\n- [Release notes](https://github.com/rust-lang/flate2-rs/releases)\n- [Commits](https://github.com/rust-lang/flate2-rs/compare/1.1.9...1.1.10)\n\nUpdates `which` from 8.0.5 to 8.0.6\n- [Release notes](https://github.com/harryfei/which-rs/releases)\n- [Changelog](https://github.com/harryfei/which-rs/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/harryfei/which-rs/compare/8.0.5...8.0.6)\n\n---\nupdated-dependencies:\n- dependency-name: async-trait\n  dependency-version: 0.1.92\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: flate2\n  dependency-version: 1.1.10\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: futures\n  dependency-version: 0.3.34\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: lru\n  dependency-version: 0.18.4\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: redb\n  dependency-version: 4.2.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-patch-minor\n- dependency-name: thiserror\n  dependency-version: 2.0.20\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: tokenizers\n  dependency-version: 0.23.2\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: toml\n  dependency-version: 1.1.5+spec-1.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: uuid\n  dependency-version: 1.26.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-patch-minor\n- dependency-name: which\n  dependency-version: 8.0.6\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-07T19:11:54+02:00",
          "tree_id": "14797b76948d60caf176adfc05bdc0acdd67f43a",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/9d91a01b83cb5e6911826fdbb1a93e364942eaf0"
        },
        "date": 1788805074810,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 42480,
            "range": "± 752",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14880,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 162350,
            "range": "± 2096",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 26926,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 130554,
            "range": "± 6156",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 40286193,
            "range": "± 2139357",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 375550733,
            "range": "± 5452259",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 181010253,
            "range": "± 4622985",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 371567822,
            "range": "± 9055058",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 75663382,
            "range": "± 2830885",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 24853323,
            "range": "± 1256051",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 167,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 451,
            "range": "± 2911",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1878390,
            "range": "± 50189",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 404,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3205,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 166,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 198,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 241,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 180,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2985,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 34106,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 353932,
            "range": "± 4824",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1803091,
            "range": "± 21844",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 157,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12157,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1122,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 321,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 352,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 397,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 322,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 494,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 657,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2261,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 66290,
            "range": "± 1243",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 246,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6611,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1141,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 32791,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 139050,
            "range": "± 1842",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 35256,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 560840,
            "range": "± 5882",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8805,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 10916517,
            "range": "± 1166750",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 10677455,
            "range": "± 651254",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 112913,
            "range": "± 2654",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 60150,
            "range": "± 1604",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4325074332,
            "range": "± 40396130",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4478576620,
            "range": "± 89753848",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4900371805,
            "range": "± 491539039",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4807747729,
            "range": "± 103148336",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4428508887,
            "range": "± 241608601",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4541539374,
            "range": "± 202767998",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4665608598,
            "range": "± 201545916",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4411454303,
            "range": "± 28211148",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4377096667,
            "range": "± 99886942",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4392526596,
            "range": "± 228270024",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4569387363,
            "range": "± 707495008",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8084533235,
            "range": "± 125409311",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4532908773,
            "range": "± 76101504",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5482719304,
            "range": "± 402051783",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6748728135,
            "range": "± 1185925881",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 28847,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 654015,
            "range": "± 7697",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 14376,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 42636,
            "range": "± 2575",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 87366,
            "range": "± 6630",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 137714,
            "range": "± 9610",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 251,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2268,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1117,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6594,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 64355,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 32339,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 19419,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 37026,
            "range": "± 5005",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 45212,
            "range": "± 3128",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1696,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 822,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 841,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 761,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 554,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 443,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6940730,
            "range": "± 1385959",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3505302,
            "range": "± 855169",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 15528,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 816,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 23840,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10677,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 48888,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 6133,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4222,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 11394,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3110,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 28577,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 277098,
            "range": "± 3148",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2788541,
            "range": "± 35220",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 178801,
            "range": "± 3614",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14133,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1061,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 338061,
            "range": "± 4935",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 31882,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2608,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 217,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 182340,
            "range": "± 2348",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 178134,
            "range": "± 2863",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 178317,
            "range": "± 2231",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 17022,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 17356,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 29014,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 40719,
            "range": "± 536",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a87d586d7859fd7166e134e4539d3a7523c1094b",
          "message": "feat(retrieval): add confidence-gated API embedding fallback for CSM (#992)\n\n* feat(retrieval): add confidence-gated API embedding fallback for CSM\n\nImplements #968: FallbackPolicy (adaptive/always_embed/local_only) with top-score and\nwinner-margin confidence gating at the Tier 4 decision point. Adaptive rescues confident\nlocal results that fail count-based sufficiency instead of counting an API call.\nCascadeResult carries fallback_reason/top_score/score_margin; eval strategies map 1:1 to\npolicies with an acceptance test (adaptive halves Tier 4 calls, no recall loss). Wave\ndesign doc records D1-D3 contracts for #966/#967/#965/#962.\n\n* test(retrieval): pin LocalOnly zero-Tier4 baseline in #968 acceptance test\n\nCovers the FallbackPolicy::LocalOnly arm in evaluate_strategy\n\nCodecov patch gap was 2 lines in eval/runner.rs. Asserts local-only\n\nevaluation reports zero embedding calls per query.\n\n---------\n\nCo-authored-by: d.o. <do-it@ik.me>\nCo-authored-by: GOAP Triage <goap-triage@local>",
          "timestamp": "2026-09-07T20:54:15+02:00",
          "tree_id": "839f48a157c86d2f1488cee07af40aa10eef4049",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/a87d586d7859fd7166e134e4539d3a7523c1094b"
        },
        "date": 1788811029768,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 60874,
            "range": "± 1117",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 22012,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 232351,
            "range": "± 1697",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 39081,
            "range": "± 640",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 193162,
            "range": "± 1587",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 110677601,
            "range": "± 69415310",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 442590701,
            "range": "± 57253805",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 223918848,
            "range": "± 31169550",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 595125056,
            "range": "± 146512744",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 103921229,
            "range": "± 16750443",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 33311500,
            "range": "± 3555575",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 168,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 305,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2388942,
            "range": "± 31981",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 494,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3363,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 184,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 292,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 408,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 229,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 4337,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 47358,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 489692,
            "range": "± 5240",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2429929,
            "range": "± 16424",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 197,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11218,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1058,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 379,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 469,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 584,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 416,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 535,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 701,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2604,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 73773,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 269,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7419,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1353,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 36841,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 198996,
            "range": "± 2107",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 49489,
            "range": "± 725",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 798264,
            "range": "± 5353",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 12172,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 12626423,
            "range": "± 1534705",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 14031045,
            "range": "± 1828755",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 105239,
            "range": "± 1133",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 69209,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4734066779,
            "range": "± 400208397",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5863453516,
            "range": "± 878916636",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4636988310,
            "range": "± 487460722",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 5112672487,
            "range": "± 821322628",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5340295857,
            "range": "± 608939638",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 5602159903,
            "range": "± 1306065027",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 6379364615,
            "range": "± 1554281535",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 5118418499,
            "range": "± 824195730",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5131150233,
            "range": "± 1160312188",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8929918059,
            "range": "± 1322249041",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5185027533,
            "range": "± 560581998",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 6477340559,
            "range": "± 1007065880",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6552546553,
            "range": "± 854427815",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 21916,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 633054,
            "range": "± 10940",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 20262,
            "range": "± 3718",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 58821,
            "range": "± 4893",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 111496,
            "range": "± 5008",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 168196,
            "range": "± 8911",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 10381466,
            "range": "± 3173232",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 288,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2603,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1369,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7455,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 73596,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 36864,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 21978,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 54528,
            "range": "± 6458",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 58109,
            "range": "± 4430",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 3142,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1230,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1203,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1114,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 688,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 530,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 8575749,
            "range": "± 2285068",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 4504501,
            "range": "± 1202920",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 13440,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1122,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 25614,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 11641,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 52833,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7548,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4984,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13957,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3441,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 30362,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 296734,
            "range": "± 2299",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 241495,
            "range": "± 5306",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 16404,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1159,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 37677,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3423,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 339,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 57743,
            "range": "± 591",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c2dc849bcafb3f61f92fb72c852dd7344c384f60",
          "message": "fix(framework): cap reranker top_k to MAX_QUERY_LIMIT (#980)\n\nClamp top_k in HierarchicalReranker::rerank_with_query with\n\nMAX_QUERY_LIMIT to prevent OOM via Vec::with_capacity. Adds\n\ntest_rerank_with_query_unbounded_top_k and a LEARNINGS entry.",
          "timestamp": "2026-09-08T18:16:54+02:00",
          "tree_id": "21b9c967a0ca7d2b4b22d84702ff03c2bda131a4",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/c2dc849bcafb3f61f92fb72c852dd7344c384f60"
        },
        "date": 1788887460805,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 47612,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18331,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 179680,
            "range": "± 532",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31747,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 157150,
            "range": "± 3828",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 42127380,
            "range": "± 1331552",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 390484710,
            "range": "± 6507683",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 195624310,
            "range": "± 3477653",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 387455608,
            "range": "± 4837800",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 79362973,
            "range": "± 1986841",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 24758363,
            "range": "± 1007701",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 329,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2233494,
            "range": "± 57768",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 476,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3668,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 194,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 233,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 282,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 209,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3680,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40613,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 430370,
            "range": "± 2058",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2206108,
            "range": "± 18438",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 188,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12835,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1231,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 389,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 433,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 485,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 403,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 547,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 732,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2800,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74921,
            "range": "± 963",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7619,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1433,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37466,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161374,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41737,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 662957,
            "range": "± 2526",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10650,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9640152,
            "range": "± 311278",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9936153,
            "range": "± 350325",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 149869,
            "range": "± 2149",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 80588,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4424204506,
            "range": "± 49616586",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4595159727,
            "range": "± 44367046",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4690545347,
            "range": "± 63180302",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5040475060,
            "range": "± 28855951",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4383799369,
            "range": "± 45597774",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4549219793,
            "range": "± 51944166",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4730314669,
            "range": "± 38668467",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4798955143,
            "range": "± 38612001",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4396784115,
            "range": "± 50082959",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4457198658,
            "range": "± 52954310",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4560800239,
            "range": "± 34625576",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8095160995,
            "range": "± 96446732",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4588068505,
            "range": "± 33698876",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5346702999,
            "range": "± 56564247",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6228707800,
            "range": "± 100651308",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 33937,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 828260,
            "range": "± 1187",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 17428,
            "range": "± 1815",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 52620,
            "range": "± 6026",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 100767,
            "range": "± 6296",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 150485,
            "range": "± 8784",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 363,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2787,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1437,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7514,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74616,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37234,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 24067,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 40833,
            "range": "± 3291",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 49485,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2219,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 975,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 973,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 932,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 662,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 521,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6172424,
            "range": "± 388282",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3074381,
            "range": "± 165244",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18709,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 906,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29484,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13170,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 59480,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7817,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5241,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14020,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3823,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35045,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 344204,
            "range": "± 2110",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 188331,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14583,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1011,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2754,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 229,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 51299,
            "range": "± 431",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "02154dfdde8e2f039b57d2a1f2b455f6ad95b2c7",
          "message": "fix(patterns): correct jaccard claims and pin duplicate semantics (#986)\n\nRoast fixes: the PR body's O(min)/O(1) mashup and blanket no-alloc claim\nare corrected (small path is O(N*M) time/O(1) space; large path still\nallocates). Remove the unreachable union_count guard (caller guarantees\na non-empty side). Add a differential test vs a set-based reference over\nempty/duplicate/asymmetric/threshold-straddling/unicode inputs plus a\nregression pin that duplicate tags stay within unit range (old code could\nexceed 1.0).\n\nCo-authored-by: d.o. <do-it@ik.me>",
          "timestamp": "2026-09-09T15:52:58+02:00",
          "tree_id": "a2c5af267f6ace4a3f398c607d9f432174d5d326",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/02154dfdde8e2f039b57d2a1f2b455f6ad95b2c7"
        },
        "date": 1788965865058,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 47729,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18013,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 178834,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31157,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 157175,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 41762447,
            "range": "± 885056",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 384881372,
            "range": "± 4368296",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 191413036,
            "range": "± 2019687",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 398179416,
            "range": "± 6826622",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 77336586,
            "range": "± 1032383",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 23755170,
            "range": "± 477232",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 295,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2280807,
            "range": "± 43916",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 387,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3677,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 194,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 233,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 284,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 209,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3718,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40824,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 434453,
            "range": "± 1731",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2229923,
            "range": "± 12361",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 187,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12823,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1233,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 388,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 424,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 474,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 404,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 547,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 738,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2778,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 75665,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7609,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1420,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37476,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161571,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41645,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 657434,
            "range": "± 2379",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10529,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9846782,
            "range": "± 192701",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9868309,
            "range": "± 249526",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 151670,
            "range": "± 3333",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 81531,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4368064623,
            "range": "± 50517729",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4502328342,
            "range": "± 38157912",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4670832028,
            "range": "± 32974394",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4988779969,
            "range": "± 53736626",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4378078012,
            "range": "± 46838810",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4503706164,
            "range": "± 30395426",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4693649079,
            "range": "± 34842885",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4740091387,
            "range": "± 53892628",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4359496001,
            "range": "± 67321968",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4458987292,
            "range": "± 47352374",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4552273601,
            "range": "± 35205720",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8051519809,
            "range": "± 93248088",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4577307439,
            "range": "± 52754195",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5329654348,
            "range": "± 57507325",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6380589008,
            "range": "± 620974924",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 33671,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 827994,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 16189,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 48695,
            "range": "± 5241",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 95989,
            "range": "± 1547",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 164374,
            "range": "± 32294",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 361,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2775,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1432,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7513,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74304,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37013,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23933,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 39432,
            "range": "± 1600",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 46988,
            "range": "± 1149",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2215,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 970,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 968,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 924,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 654,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 518,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5930051,
            "range": "± 123093",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3004116,
            "range": "± 66077",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18699,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 927,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29258,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13178,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 59552,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7668,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5275,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14166,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3808,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35154,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 344708,
            "range": "± 1913",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 188117,
            "range": "± 1540",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14456,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1101,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 32510,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3173,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 391,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50840,
            "range": "± 438",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d79c2570263232a0c3b8dba0993e6ed1cdb5160e",
          "message": "feat(storage): batch durable episode writes off the completion path (#993)\n\nImplements #967 behind opt-in MemoryConfig.durable_write_queue (default None preserves\nall-synchronous ADR-075 behavior). Local cache writes stay synchronous and hard-error;\nTurso persists via a single ordered background worker in transactional batches with\nretry/backoff. Same-episode writes coalesce (seam + sync pattern-path re-store collapse\nto one remote write). Backpressure is explicit QuotaExceeded collected into the Storage\nerror; retries are idempotent via INSERT OR REPLACE. New flush_durable_writes/drain API,\nWriteQueueStats, and EpisodeComplete journal rows; StorageBackend gains\nstore_episodes_batch with a looping default (Turso overrides transactionally). 7\nintegration tests: off-path latency, flush failure surfacing, retry-no-dup, backpressure,\ndrain-on-shutdown, no-turso passthrough, failure-ID cap.\n\nCo-authored-by: d.o. <do-it@ik.me>",
          "timestamp": "2026-09-09T18:20:48+02:00",
          "tree_id": "d7c4fd55a14230440e1b37d54dca77781ce165f9",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/d79c2570263232a0c3b8dba0993e6ed1cdb5160e"
        },
        "date": 1788974901542,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48058,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18295,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 177885,
            "range": "± 958",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31205,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156582,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 41490099,
            "range": "± 958166",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 391010432,
            "range": "± 9158515",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 196697484,
            "range": "± 6140849",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 405999373,
            "range": "± 13769490",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 80839273,
            "range": "± 1974190",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 25506439,
            "range": "± 971272",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 277,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2288466,
            "range": "± 48220",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 525,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3648,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 195,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 234,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 283,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 209,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3687,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 41211,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 431990,
            "range": "± 2222",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2203475,
            "range": "± 15842",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 188,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12830,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1237,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 394,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 449,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 482,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 410,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 549,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 736,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2794,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74833,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 332,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7541,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1420,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37322,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 160716,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41210,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 661784,
            "range": "± 4589",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10644,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 10114980,
            "range": "± 239765",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9707717,
            "range": "± 285795",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 149986,
            "range": "± 2385",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 81874,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4359317174,
            "range": "± 52290962",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4531273291,
            "range": "± 39660547",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4703305308,
            "range": "± 45322849",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5006632020,
            "range": "± 41244456",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4367796204,
            "range": "± 35891062",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4554325280,
            "range": "± 70155207",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4677380789,
            "range": "± 44969545",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4742566741,
            "range": "± 44416310",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4366410079,
            "range": "± 59776250",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4430095459,
            "range": "± 49432006",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4553219843,
            "range": "± 42274320",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8074443862,
            "range": "± 104459347",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4591681178,
            "range": "± 52832654",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5318243788,
            "range": "± 58003345",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6184218448,
            "range": "± 61877517",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34085,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 866179,
            "range": "± 42658",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 17727,
            "range": "± 1890",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 49745,
            "range": "± 4525",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 121550,
            "range": "± 29168",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 153802,
            "range": "± 10023",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 338,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2778,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1440,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7540,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74594,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37306,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23361,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 42363,
            "range": "± 3128",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 50509,
            "range": "± 3175",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2217,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1083,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1094,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1040,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 665,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 519,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5821891,
            "range": "± 241760",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3089401,
            "range": "± 229532",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18791,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 931,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29838,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13334,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 62158,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7490,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4973,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13737,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3900,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35719,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349108,
            "range": "± 8412",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14894,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 989,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3493,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 311,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 52372,
            "range": "± 603",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "d-oit",
            "username": "d-oit"
          },
          "committer": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "Dominik Oswald",
            "username": "d-oit"
          },
          "distinct": true,
          "id": "2345b53586de5717d0518d4b0cff68cb479f8445",
          "message": "feat(handoff): address PR #994 review feedback and refine compact handoffs\n\n- Fix byte-budget token convergence off-by-metadata bug in assemble_compact.\n- Decouple full-context recovery guidance from core OmissionMetadata.\n- Account for outer GetHandoffPackOutput wrapper overhead in MCP budget.\n- Enhance human CLI output to print pattern/heuristic/artifact refs and truncation warnings.\n- Enforce pure snapshot semantics as of checkpoint.step_number across all handoff fields.\n- Add artifact_refs and multi-step workflow resume quality integration test.",
          "timestamp": "2026-09-11T12:31:03+02:00",
          "tree_id": "1192fdc91a9a2fd7775f852dadcfa7e498ca9cae",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/2345b53586de5717d0518d4b0cff68cb479f8445"
        },
        "date": 1789125806850,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 39324,
            "range": "± 1127",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 13511,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 153019,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25517,
            "range": "± 3089",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 124707,
            "range": "± 631",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 52879721,
            "range": "± 15357043",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 688706446,
            "range": "± 224013390",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 269296425,
            "range": "± 110337119",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 801146817,
            "range": "± 670440896",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 124360225,
            "range": "± 54135331",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 97493702,
            "range": "± 77220862",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 120,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 198,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1202552,
            "range": "± 101427",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 289,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2501,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 213,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 170,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2855,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 31572,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 330109,
            "range": "± 5153",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1626475,
            "range": "± 14445",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 134,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 7906,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 764,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 246,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 316,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 398,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 278,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 368,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 488,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_5000",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 1485,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 52219,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 176,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 5314,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 735,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 26230,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 128383,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 32456,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 518263,
            "range": "± 2923",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8030,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 17767767,
            "range": "± 9634958",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 15349690,
            "range": "± 10222877",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 79085,
            "range": "± 2018",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 45426,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 6552820276,
            "range": "± 1483995894",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 7989678848,
            "range": "± 5545405803",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 6163199697,
            "range": "± 1092305815",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5510855355,
            "range": "± 1222498516",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5260680315,
            "range": "± 762912060",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 5787330681,
            "range": "± 2586477902",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4858475537,
            "range": "± 779861559",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8650076209,
            "range": "± 592606741",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 6215953312,
            "range": "± 1410896481",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 9402055586,
            "range": "± 5511740431",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6792118541,
            "range": "± 804354540",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 21561,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_1536",
            "value": 898,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_3072",
            "value": 1801,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_384",
            "value": 225,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_768",
            "value": 449,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_simd_1536",
            "value": 895,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_simd_384",
            "value": 226,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_simd_768",
            "value": 448,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 384078,
            "range": "± 4140",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 10511,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 31213,
            "range": "± 2556",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 61927,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 94764,
            "range": "± 2585",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 17223218,
            "range": "± 14339128",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 30101331,
            "range": "± 23096058",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 83724115,
            "range": "± 58086097",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 190,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 1487,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 744,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 5310,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 52244,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 26145,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 16260,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 25959,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 31005,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1122,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 826,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 807,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 734,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 445,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 352,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 12554890,
            "range": "± 10996727",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 6823423,
            "range": "± 6457902",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 10128,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 754,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100000_k10000",
            "value": 2130289,
            "range": "± 118951",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n10000_k1000",
            "value": 185576,
            "range": "± 269302",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 26023,
            "range": "± 105347",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1068,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 17617,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 7644,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 36791,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 4505,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 2956,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 8552,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "summary_key_concept_extraction_5",
            "value": 1734,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 1796,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 15147,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 146035,
            "range": "± 1568",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 1789668,
            "range": "± 6792",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 105201,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 8566,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 644,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 202682,
            "range": "± 979",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 18688,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 1681,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 250,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 105382,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 105630,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 105717,
            "range": "± 4509",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 106495,
            "range": "± 6824",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 8665,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 9388,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 17927,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 60896,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 31636,
            "range": "± 197",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dccef98856fe0c9bf6a52b7304f64cd6b60c6c23",
          "message": "perf(patterns): optimize string similarity with ASCII fast-path and stack DP (#1010)\n\nIntroduced stack-buffered slice_edit_distance and an ASCII fast path in\nstring_similarity and edit_distance within do-memory-core. For ASCII strings\nup to 128 characters, heap allocations drop from 2 to 0.\n\nCo-authored-by: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>\nCo-authored-by: Dominik Oswald <6849456+d-oit@users.noreply.github.com>",
          "timestamp": "2026-09-13T21:06:13+02:00",
          "tree_id": "36f48c5ead15215093a09c55c08aec6137488c9a",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/dccef98856fe0c9bf6a52b7304f64cd6b60c6c23"
        },
        "date": 1789331007814,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 40449,
            "range": "± 1066",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14164,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 158026,
            "range": "± 4194",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25653,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 127626,
            "range": "± 3077",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 146664876,
            "range": "± 148161912",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 1006322148,
            "range": "± 497178363",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 554652292,
            "range": "± 244105593",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 674292425,
            "range": "± 147967637",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 185032666,
            "range": "± 84893157",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 54729769,
            "range": "± 37778834",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 122,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 205,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 24,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1232685,
            "range": "± 126202",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 367,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2522,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 140,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 216,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 309,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 175,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2964,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 31558,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 331889,
            "range": "± 5142",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1649660,
            "range": "± 51163",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 135,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 8156,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 781,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 253,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 328,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 419,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 290,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 387,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 508,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 43,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 42,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_5000",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 1554,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 53690,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 172,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 5392,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 776,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 27449,
            "range": "± 1309",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 130222,
            "range": "± 2482",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 33534,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 525524,
            "range": "± 15780",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8211,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 65243358,
            "range": "± 56587998",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 32478171,
            "range": "± 18407190",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 81138,
            "range": "± 1952",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 48002,
            "range": "± 1841",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 8845378448,
            "range": "± 2574085424",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 8606458530,
            "range": "± 1634939456",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 13104462046,
            "range": "± 2868255984",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 18037372236,
            "range": "± 4022571178",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 9430892861,
            "range": "± 2177679413",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 10091840517,
            "range": "± 2978578832",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 8938833511,
            "range": "± 2043509139",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 21869,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_384",
            "value": 238,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_scalar_768",
            "value": 469,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_simd_384",
            "value": 234,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cosine_similarity_simd_768",
            "value": 462,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 409260,
            "range": "± 5639",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 25030,
            "range": "± 36596",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 64896,
            "range": "± 81293",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 87842,
            "range": "± 63831",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 152525,
            "range": "± 69128",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 57977439,
            "range": "± 55059682",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 42829596,
            "range": "± 33159206",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_100",
            "value": 220318177,
            "range": "± 247999420",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 151954580,
            "range": "± 135370701",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 194,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 1529,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 769,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 5500,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 53329,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 26262,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 17129,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 44382,
            "range": "± 43825",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 48514,
            "range": "± 33488",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1174,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 816,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 817,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 746,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 467,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 359,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 33381311,
            "range": "± 45696589",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 10207218,
            "range": "± 14909045",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 10845,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 775,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100000_k10000",
            "value": 2201208,
            "range": "± 289017",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n10000_k1000",
            "value": 227859,
            "range": "± 498011",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 22616,
            "range": "± 61861",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1071,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 18061,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 7956,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 37397,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 49,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 4740,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 2996,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 8686,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "summary_key_concept_extraction_5",
            "value": 1812,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 1989,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 16592,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 155925,
            "range": "± 3289",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 1867632,
            "range": "± 46719",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 107193,
            "range": "± 2134",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 8664,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 643,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 195756,
            "range": "± 4576",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 21132,
            "range": "± 888",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2039,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 156,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 108164,
            "range": "± 2277",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 110031,
            "range": "± 4339",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 112829,
            "range": "± 1528",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 109197,
            "range": "± 4764",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 10155,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 10931,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 20341,
            "range": "± 2248",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 63551,
            "range": "± 2545",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 33624,
            "range": "± 526",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "committer": {
            "email": "242170972+d-o-hub@users.noreply.github.com",
            "name": "d.o.",
            "username": "d-o-hub"
          },
          "distinct": true,
          "id": "e4a627484796ce3802e575c1c19e64587fa0c73f",
          "message": "fix(ci): stop committing generated coverage reports",
          "timestamp": "2026-09-13T23:17:57+02:00",
          "tree_id": "981a401156d982fd1ba9563d40e319a9e3cbc468",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/e4a627484796ce3802e575c1c19e64587fa0c73f"
        },
        "date": 1789337360472,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 54348,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 19798,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 207101,
            "range": "± 2995",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 34976,
            "range": "± 600",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 174576,
            "range": "± 4867",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 41828548,
            "range": "± 1329802",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 445311596,
            "range": "± 208293098",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 250782799,
            "range": "± 37849672",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 377081864,
            "range": "± 10444886",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 79127508,
            "range": "± 4436945",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 27258738,
            "range": "± 1101563",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 150,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 376,
            "range": "± 2151",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2166129,
            "range": "± 36773",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 387,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2997,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 166,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 260,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 365,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 206,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3866,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 42624,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 441843,
            "range": "± 12640",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2183434,
            "range": "± 44184",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 176,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 10055,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 953,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 340,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 429,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 528,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 373,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 486,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 619,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2348,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 66352,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 237,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6732,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1212,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 33089,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 177669,
            "range": "± 3249",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 44347,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 714443,
            "range": "± 11609",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10818,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 11673904,
            "range": "± 932170",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 12134089,
            "range": "± 1331534",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 96740,
            "range": "± 2775",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 62055,
            "range": "± 1162",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 5358362695,
            "range": "± 996750639",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4898257915,
            "range": "± 545230841",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5545363205,
            "range": "± 1038796411",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 5469007200,
            "range": "± 1325413812",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4782047362,
            "range": "± 551541273",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5164552023,
            "range": "± 870289607",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4427228145,
            "range": "± 75842115",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4511275099,
            "range": "± 361651737",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4623541341,
            "range": "± 570907409",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4620901319,
            "range": "± 568001855",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 9295525475,
            "range": "± 1343755207",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4633507958,
            "range": "± 398224975",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5420950429,
            "range": "± 387348310",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6482346754,
            "range": "± 615774913",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 19900,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 555623,
            "range": "± 12051",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 17257,
            "range": "± 1986",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 52149,
            "range": "± 3146",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 103646,
            "range": "± 6154",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 159006,
            "range": "± 8501",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 7911346,
            "range": "± 568016",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 252,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2313,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1221,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6655,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 65872,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 33077,
            "range": "± 721",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 19265,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 48824,
            "range": "± 5000",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 52006,
            "range": "± 3265",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2766,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1088,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1091,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 980,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 623,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 477,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 7635466,
            "range": "± 1952249",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 4864962,
            "range": "± 4927231",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 12015,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 988,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 24420,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10962,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 49481,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 6792,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4298,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 12105,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 2870,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 24738,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 241608,
            "range": "± 1445",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2918896,
            "range": "± 124687",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 213700,
            "range": "± 3064",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14602,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1070,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 383222,
            "range": "± 2343",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 34297,
            "range": "± 708",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2654,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 264,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 214629,
            "range": "± 1475",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 216744,
            "range": "± 9828",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 17940,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 18893,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 32584,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50885,
            "range": "± 260",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "d.o.",
            "username": "d-o-hub",
            "email": "242170972+d-o-hub@users.noreply.github.com"
          },
          "committer": {
            "name": "d.o.",
            "username": "d-o-hub",
            "email": "242170972+d-o-hub@users.noreply.github.com"
          },
          "id": "e4a627484796ce3802e575c1c19e64587fa0c73f",
          "message": "fix(ci): stop committing generated coverage reports",
          "timestamp": "2026-09-13T18:17:38Z",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/e4a627484796ce3802e575c1c19e64587fa0c73f"
        },
        "date": 1789356248966,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 47639,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18019,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 178901,
            "range": "± 1204",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31101,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 158297,
            "range": "± 9095",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 45029051,
            "range": "± 1704137",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 393646516,
            "range": "± 7960157",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 195349885,
            "range": "± 4273468",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 396449182,
            "range": "± 17465484",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 81856999,
            "range": "± 4026609",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 24657344,
            "range": "± 382034",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 277,
            "range": "± 971",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2337930,
            "range": "± 49390",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 475,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3677,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 196,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 233,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 209,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3735,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40646,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 433734,
            "range": "± 4950",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2234465,
            "range": "± 88313",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 188,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12836,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1242,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 393,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 435,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 488,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 415,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 548,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 734,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2802,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 75293,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 331,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7561,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1428,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37293,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 162523,
            "range": "± 8652",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41883,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 662791,
            "range": "± 8759",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10545,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9978280,
            "range": "± 363568",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9777094,
            "range": "± 378618",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 150741,
            "range": "± 1720",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 82179,
            "range": "± 657",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4408215634,
            "range": "± 56100559",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4590196122,
            "range": "± 138275006",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4704689741,
            "range": "± 83484437",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5113105369,
            "range": "± 65366263",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4447928890,
            "range": "± 112797816",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4601723899,
            "range": "± 52754521",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4805967171,
            "range": "± 89115970",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4796139862,
            "range": "± 37160308",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4417480048,
            "range": "± 91705184",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4447422373,
            "range": "± 51811628",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4585389204,
            "range": "± 53482936",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8099320146,
            "range": "± 114501243",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4616765255,
            "range": "± 49852672",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5387164832,
            "range": "± 79240409",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6251077769,
            "range": "± 52906083",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34716,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 858091,
            "range": "± 10374",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 20363,
            "range": "± 3956",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 57372,
            "range": "± 10047",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 114143,
            "range": "± 22392",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 153314,
            "range": "± 5724",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 371,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2789,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1430,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7570,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 75405,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37486,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23987,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 43120,
            "range": "± 4648",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 52482,
            "range": "± 5381",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2218,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1010,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1015,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 963,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 676,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 532,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6012342,
            "range": "± 316521",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3068396,
            "range": "± 192871",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18746,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 927,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29793,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13133,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 61681,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7374,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5010,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13739,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3895,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35690,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349395,
            "range": "± 5793",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14634,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1084,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 388,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 52277,
            "range": "± 177",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "22c47b39cd5c8540147423e4e3c7769d979587db",
          "message": "chore(deps): bump the rust-major group with 3 updates (#1019)\n\nBumps the rust-major group with 3 updates: dirs, [zstd](https://github.com/gyscos/zstd-rs) and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).\n\n\nUpdates `dirs` from 6.0.0 to 7.0.0\n\nUpdates `zstd` from 0.13.3 to 0.14.0\n- [Release notes](https://github.com/gyscos/zstd-rs/releases)\n- [Commits](https://github.com/gyscos/zstd-rs/compare/v0.13.3...v0.14.0)\n\nUpdates `sysinfo` from 0.38.4 to 0.39.6\n- [Changelog](https://github.com/GuillaumeGomez/sysinfo/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/GuillaumeGomez/sysinfo/compare/v0.38.4...v0.39.6)\n\n---\nupdated-dependencies:\n- dependency-name: dirs\n  dependency-version: 7.0.0\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n  dependency-group: rust-major\n- dependency-name: zstd\n  dependency-version: 0.14.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-major\n- dependency-name: sysinfo\n  dependency-version: 0.39.6\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-14T19:07:29+02:00",
          "tree_id": "c7288c34cd857e138a5f20c2b6ab5143a1c53f5d",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/22c47b39cd5c8540147423e4e3c7769d979587db"
        },
        "date": 1789408757830,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 39999,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14384,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 152590,
            "range": "± 514",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25739,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 122235,
            "range": "± 2631",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 53015977,
            "range": "± 17911123",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 565463209,
            "range": "± 157489970",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 206067584,
            "range": "± 30353542",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 409199706,
            "range": "± 124167911",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 109946470,
            "range": "± 52394960",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 36357387,
            "range": "± 20700277",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 153,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 290,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1648971,
            "range": "± 100666",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 487,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2989,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 156,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 192,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 232,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 177,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2931,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 33302,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 357056,
            "range": "± 3259",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1811587,
            "range": "± 10724",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 151,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11227,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1063,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 313,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 342,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 384,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 324,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 461,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 619,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_5000",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2198,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 63582,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 238,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6485,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1107,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 31844,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 125385,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 33143,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 512345,
            "range": "± 17684",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8283,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 13140879,
            "range": "± 7660256",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 16659005,
            "range": "± 9629879",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 105965,
            "range": "± 2568",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 58168,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5428751772,
            "range": "± 569006304",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 6712621389,
            "range": "± 2117425150",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 5791439165,
            "range": "± 982565136",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 6374101165,
            "range": "± 1558542433",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5925407348,
            "range": "± 1328469524",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 6296321674,
            "range": "± 1559495132",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 7620131272,
            "range": "± 1680554207",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 9726913921,
            "range": "± 1949665955",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5154501498,
            "range": "± 654684747",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5953230806,
            "range": "± 634927732",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 7182009784,
            "range": "± 1050892559",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 27651,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 629356,
            "range": "± 5622",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 14225,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 42653,
            "range": "± 4143",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 81386,
            "range": "± 5199",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 136224,
            "range": "± 20263",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 240,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2185,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1104,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6474,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 63660,
            "range": "± 627",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 31841,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 18826,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 34237,
            "range": "± 3378",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 42089,
            "range": "± 4858",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1561,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 835,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 822,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 761,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 522,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 414,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 11847275,
            "range": "± 11341751",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3505334,
            "range": "± 1114215",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14780,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 774,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1124,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 23405,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10535,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 49248,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 5959,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4008,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 11011,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3018,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 27595,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 268095,
            "range": "± 1781",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2607062,
            "range": "± 4215",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 164625,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 12867,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 905,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 314687,
            "range": "± 16651",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 25598,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2398,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 247,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 165246,
            "range": "± 764",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 165799,
            "range": "± 2629",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 165851,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 165993,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 15690,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 16387,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 28812,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 91704,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 40509,
            "range": "± 141",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}