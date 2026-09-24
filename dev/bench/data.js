window.BENCHMARK_DATA = {
  "lastUpdate": 1790272755572,
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
          "id": "2fe9d294e04c5e4cf92d6cfd767f3ddcaae3b256",
          "message": "chore(deps): bump the rust-patch-minor group with 3 updates (#1018)\n\nBumps the rust-patch-minor group with 3 updates: [uuid](https://github.com/uuid-rs/uuid), [toml](https://github.com/toml-rs/toml) and [reqwest](https://github.com/seanmonstar/reqwest).\n\n\nUpdates `uuid` from 1.26.0 to 1.26.1\n- [Release notes](https://github.com/uuid-rs/uuid/releases)\n- [Commits](https://github.com/uuid-rs/uuid/compare/v1.26.0...v1.26.1)\n\nUpdates `toml` from 1.1.5+spec-1.1.0 to 1.1.6+spec-1.1.0\n- [Commits](https://github.com/toml-rs/toml/compare/toml-v1.1.5...toml-v1.1.6)\n\nUpdates `reqwest` from 0.13.4 to 0.13.5\n- [Release notes](https://github.com/seanmonstar/reqwest/releases)\n- [Changelog](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/seanmonstar/reqwest/compare/v0.13.4...v0.13.5)\n\n---\nupdated-dependencies:\n- dependency-name: uuid\n  dependency-version: 1.26.1\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: toml\n  dependency-version: 1.1.6+spec-1.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: reqwest\n  dependency-version: 0.13.5\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-14T18:16:08Z",
          "tree_id": "4ca0c55b607874983f8decb71a8376a7e5eea750",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/2fe9d294e04c5e4cf92d6cfd767f3ddcaae3b256"
        },
        "date": 1789412882577,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 61817,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 22525,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 235954,
            "range": "± 828",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 39946,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 198097,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 59856776,
            "range": "± 6080595",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 494488104,
            "range": "± 42088459",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 274958237,
            "range": "± 46013142",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 520503029,
            "range": "± 31103239",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 104712107,
            "range": "± 6895734",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 37875242,
            "range": "± 2669857",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 173,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 315,
            "range": "± 1307",
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
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2471147,
            "range": "± 24225",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 490,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3465,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 189,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 298,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 422,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 237,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 4369,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 47573,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 493005,
            "range": "± 2287",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2482105,
            "range": "± 16285",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 194,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11635,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1095,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 381,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 484,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 598,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 429,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 554,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 725,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2674,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 76217,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 277,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7723,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1402,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 38279,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 204384,
            "range": "± 2533",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 50792,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 824942,
            "range": "± 9746",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 12181,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 13738888,
            "range": "± 1661224",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 12789520,
            "range": "± 1126276",
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
            "value": 106655,
            "range": "± 2942",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 71533,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 6288736540,
            "range": "± 425128042",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 5408961407,
            "range": "± 293837292",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 5491149639,
            "range": "± 175174456",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5550503592,
            "range": "± 131036742",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 5675962061,
            "range": "± 391586897",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5270281733,
            "range": "± 239692875",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 5379467125,
            "range": "± 174016909",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5516231764,
            "range": "± 324354596",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 10018619479,
            "range": "± 442854616",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5678144223,
            "range": "± 228058747",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 6420152784,
            "range": "± 364126023",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 7798670939,
            "range": "± 502949672",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 23332,
            "range": "± 1448",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 660891,
            "range": "± 13835",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 19603,
            "range": "± 3149",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 58296,
            "range": "± 4310",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 117186,
            "range": "± 7588",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 172916,
            "range": "± 9291",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 293,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2685,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1390,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7723,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 76089,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 38071,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 22471,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 56295,
            "range": "± 7955",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 58725,
            "range": "± 5502",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 3176,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1258,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1264,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1142,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 695,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 531,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 8249147,
            "range": "± 1592838",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 4663462,
            "range": "± 914925",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14022,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1149,
            "range": "± 3",
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
            "value": 26707,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12351,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 55454,
            "range": "± 591",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7912,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5122,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14347,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3341,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 29302,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 287430,
            "range": "± 3303",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 17038,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1287,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3403,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 239,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 59802,
            "range": "± 408",
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
          "id": "f3101929449ed0376712db5300b58729b185671b",
          "message": "fix(deps): bump rustls to 0.23.45 (RUSTSEC-2026-0285) (#1021)",
          "timestamp": "2026-09-15T11:23:44+02:00",
          "tree_id": "fb32151176ff33c3c5ddc7bfee763bede1269042",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/f3101929449ed0376712db5300b58729b185671b"
        },
        "date": 1789467325128,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 40226,
            "range": "± 1761",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 13864,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 154880,
            "range": "± 4052",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25561,
            "range": "± 1140",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 128311,
            "range": "± 5934",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 347533208,
            "range": "± 293450274",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 1955120290,
            "range": "± 1213159661",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 1046935885,
            "range": "± 746342370",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 2689322369,
            "range": "± 1123427526",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 393420820,
            "range": "± 196211559",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 139568214,
            "range": "± 60266926",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 124,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 205,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1165894,
            "range": "± 50479",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 481,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2511,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 137,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 218,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 314,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 172,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3010,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 32526,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 334502,
            "range": "± 18134",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1673525,
            "range": "± 34222",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 137,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 8273,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 798,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 259,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 326,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 412,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 286,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 383,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 498,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 42,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 42,
            "range": "± 1",
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
            "value": 1516,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 53908,
            "range": "± 1349",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 175,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 5558,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 756,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 27086,
            "range": "± 1213",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 132497,
            "range": "± 3813",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 33934,
            "range": "± 2088",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 525773,
            "range": "± 8605",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8271,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 58095968,
            "range": "± 83858932",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 33901663,
            "range": "± 30815472",
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
            "value": 81439,
            "range": "± 3491",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 48374,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 15622272640,
            "range": "± 3951585829",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 19892600655,
            "range": "± 7924959699",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 20666671538,
            "range": "± 5335586644",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 22595,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 392139,
            "range": "± 17482",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 11204,
            "range": "± 2069",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 32426,
            "range": "± 2613",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 63467,
            "range": "± 5395",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 100543,
            "range": "± 8123",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 30533805,
            "range": "± 27074386",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 33732117,
            "range": "± 30706027",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 121536180,
            "range": "± 145443399",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 186,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 1521,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 786,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 5496,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 53704,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 28036,
            "range": "± 2382",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 16845,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 27611,
            "range": "± 2643",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 33787,
            "range": "± 7626",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1154,
            "range": "± 22",
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
            "value": 829,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 835,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 766,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 453,
            "range": "± 35",
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
            "value": 45029586,
            "range": "± 75995561",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 21334879,
            "range": "± 36671772",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 10572,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 759,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100000_k10000",
            "value": 2232084,
            "range": "± 136630",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n10000_k1000",
            "value": 197279,
            "range": "± 264724",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 23281,
            "range": "± 64033",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1080,
            "range": "± 33",
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
            "value": 17112,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 7517,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 36662,
            "range": "± 3228",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 49,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 4641,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 3178,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 9070,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "summary_key_concept_extraction_5",
            "value": 1743,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 1790,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 16038,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 148067,
            "range": "± 4319",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 1838707,
            "range": "± 59978",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 109063,
            "range": "± 2665",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 8613,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 638,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 203699,
            "range": "± 2430",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 17723,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 1755,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 292,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 112043,
            "range": "± 6692",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 114616,
            "range": "± 8199",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 113162,
            "range": "± 2278",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 110797,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 9219,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 9758,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 19873,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 65024,
            "range": "± 5463",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 32460,
            "range": "± 702",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Dominik Oswald",
            "username": "d-oit",
            "email": "6849456+d-oit@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5893cfd2431572ad2ff3da4a73886cebb2e5d1d2",
          "message": "feat(observability): expose retrieval metrics in MCP tools and add acceptance tests (#962) (#1005)\n\n* feat(observability): expose retrieval metrics in MCP tools and add acceptance tests (#962)\n\n* test(retrieval): rename acceptance test to permanent retrieval_metrics_test\n\n---------\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-15T11:14:20Z",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/5893cfd2431572ad2ff3da4a73886cebb2e5d1d2"
        },
        "date": 1789471809113,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 55621,
            "range": "± 3106",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 19952,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 213044,
            "range": "± 9234",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 35692,
            "range": "± 1253",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 176487,
            "range": "± 6061",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 166336635,
            "range": "± 68167242",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 818491322,
            "range": "± 538473985",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 538718356,
            "range": "± 258557728",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 912251954,
            "range": "± 451165382",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 269562518,
            "range": "± 197585348",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 76886366,
            "range": "± 33237516",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 153,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 394,
            "range": "± 2277",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 28,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2375919,
            "range": "± 111652",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 422,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3089,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 169,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 275,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 382,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 224,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3998,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 43577,
            "range": "± 1771",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 443631,
            "range": "± 12072",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2226028,
            "range": "± 93072",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 189,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 10391,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 975,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 351,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 437,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 534,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 391,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 491,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 629,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2408,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 67993,
            "range": "± 2441",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 265,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6745,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1219,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 33867,
            "range": "± 1600",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 180456,
            "range": "± 5175",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 45953,
            "range": "± 1873",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 731760,
            "range": "± 21960",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 11356,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 22492433,
            "range": "± 21234715",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 29319847,
            "range": "± 23087377",
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
            "value": 98282,
            "range": "± 2514",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 64803,
            "range": "± 2539",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 11134025577,
            "range": "± 2187259602",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 15266051278,
            "range": "± 4557129719",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 14508298392,
            "range": "± 3717040277",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 7014579332,
            "range": "± 2006187649",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 8042324857,
            "range": "± 2413173895",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 14394262266,
            "range": "± 4849853539",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 20666,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 584221,
            "range": "± 22816",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 18141,
            "range": "± 2666",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 53120,
            "range": "± 3876",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 108420,
            "range": "± 7955",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 157735,
            "range": "± 13471",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 16631423,
            "range": "± 12111799",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 43073972,
            "range": "± 33021926",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 50492144,
            "range": "± 38503497",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 262,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2397,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1239,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6790,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 68122,
            "range": "± 2896",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 33655,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 20541,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 50072,
            "range": "± 5118",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 52846,
            "range": "± 4427",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2847,
            "range": "± 99",
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
            "value": 1123,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1118,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1009,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 623,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 471,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 12400356,
            "range": "± 10026161",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 11705079,
            "range": "± 15177577",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 12263,
            "range": "± 438",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1012,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 23514,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10837,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 48073,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 68,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 6981,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4674,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 12608,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 2924,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 25902,
            "range": "± 1172",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 251899,
            "range": "± 12519",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2953129,
            "range": "± 83435",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 223843,
            "range": "± 8831",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 15546,
            "range": "± 624",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1070,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 35398,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3347,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 387,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 52188,
            "range": "± 1624",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "Dominik Oswald",
            "username": "d-oit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5893cfd2431572ad2ff3da4a73886cebb2e5d1d2",
          "message": "feat(observability): expose retrieval metrics in MCP tools and add acceptance tests (#962) (#1005)\n\n* feat(observability): expose retrieval metrics in MCP tools and add acceptance tests (#962)\n\n* test(retrieval): rename acceptance test to permanent retrieval_metrics_test\n\n---------\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-15T13:14:20+02:00",
          "tree_id": "df7d4784dfeb728ecf5f154c0708605294cb061d",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/5893cfd2431572ad2ff3da4a73886cebb2e5d1d2"
        },
        "date": 1789474004593,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48251,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18000,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 182234,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31645,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156639,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 39466264,
            "range": "± 624966",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 381487405,
            "range": "± 4753543",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 186745207,
            "range": "± 1815798",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 382286059,
            "range": "± 4462412",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 76357649,
            "range": "± 2481509",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 22982181,
            "range": "± 184849",
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
            "value": 275,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2216271,
            "range": "± 63346",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 396,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3702,
            "range": "± 50",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 285,
            "range": "± 0",
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
            "value": 3685,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40972,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 432064,
            "range": "± 3011",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2200162,
            "range": "± 14950",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 184,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12925,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1235,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 390,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 435,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 507,
            "range": "± 31",
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
            "value": 551,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 734,
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
            "value": 2775,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74405,
            "range": "± 303",
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
            "value": 7554,
            "range": "± 57",
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
            "value": 37146,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161156,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41478,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 658764,
            "range": "± 3164",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10556,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9168885,
            "range": "± 136881",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 8936862,
            "range": "± 119370",
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
            "value": 149660,
            "range": "± 1356",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 80965,
            "range": "± 640",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4355405484,
            "range": "± 12429843",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4498404555,
            "range": "± 53478694",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4664805869,
            "range": "± 32746030",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4962117930,
            "range": "± 45893453",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4356911129,
            "range": "± 48121553",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4499102866,
            "range": "± 45621705",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4656169565,
            "range": "± 33863724",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4710730223,
            "range": "± 57881344",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4353917913,
            "range": "± 29706796",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4433055764,
            "range": "± 40535178",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4534558826,
            "range": "± 46972429",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8089507039,
            "range": "± 124229594",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4544614489,
            "range": "± 67517680",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5267961660,
            "range": "± 45161481",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6173859765,
            "range": "± 80472891",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34785,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 846038,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 16970,
            "range": "± 1458",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 48016,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 101941,
            "range": "± 5984",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 150488,
            "range": "± 5741",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 342,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2766,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1424,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7535,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74941,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37128,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23400,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 40422,
            "range": "± 1556",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 48201,
            "range": "± 2717",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2210,
            "range": "± 6",
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
            "value": 987,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 986,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 943,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 664,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 529,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5549990,
            "range": "± 97144",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2786629,
            "range": "± 46381",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18821,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 915,
            "range": "± 2",
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
            "value": 28776,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13060,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 58533,
            "range": "± 710",
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
            "value": 7579,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5125,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13748,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3915,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35487,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349426,
            "range": "± 1513",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 188010,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14816,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 995,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3039,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 223,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 52294,
            "range": "± 235",
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
          "id": "04744db9e60fbeb0c75c4f64a3a02c41957988af",
          "message": "fix(ci): ingest only fresh Criterion new/ estimates in benchmarks (#1022)\n\nThe conversion step read every estimates.json under the criterion dir and\nstripped /new/, /base/ and /change/ down to a single benchmark name, emitting\nall of them as absolute ns. A scratch Criterion 0.8 bench shows base/ is a\nbyte-identical copy of new/ and change/ holds RELATIVE deltas (e.g. -0.099),\nwhich the old code emitted as \"-1 ns/iter\". CI evidence: run 34879770281\n(232 files -> 222 lines -> 111 entries) and the PR #1005 run\n(233 files -> 223 lines -> 112 entries, with 5 compression_overhead/\nwithout_compression benchmarks silently dropped because floor(0.311 ns) = 0).\n\n- restrict ingestion to -path '*/new/estimates.json'\n- emit 2-decimal floats instead of flooring, so sub-ns means survive; the\n  action's cargo parser accepts them ([0-9,.]+ value/range groups)\n- fail closed on estimates with no positive ns mean (was a silent drop)\n- fail closed when converted rows != fresh file count (provenance)\n- fail closed on duplicate benchmark names after conversion\n- extend scripts/test-benchmark-workflow.sh fixtures (sub-ns + freshness)\n- record LESSON-025 and the ingest rule in github_actions_patterns.md",
          "timestamp": "2026-09-15T14:25:07+02:00",
          "tree_id": "3f7480b90b2481c758dbcda5a1754b8bc2d69273",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/04744db9e60fbeb0c75c4f64a3a02c41957988af"
        },
        "date": 1789478217757,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 39892.23,
            "range": "± 179.17",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14138.63,
            "range": "± 70.82",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 153806.66,
            "range": "± 1068.63",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25409.71,
            "range": "± 177.38",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 121633.55,
            "range": "± 1478.31",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 35830321.97,
            "range": "± 732889.99",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 373490706.5,
            "range": "± 5255134.85",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 182386193,
            "range": "± 5052987.03",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 374139053.15,
            "range": "± 3393121.27",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 69764410.2,
            "range": "± 960448.99",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 21464471.06,
            "range": "± 497730.03",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 152.55,
            "range": "± 0.77",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 292.39,
            "range": "± 1477.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 8.74,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 16.43,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1609117.79,
            "range": "± 89578.22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 386.42,
            "range": "± 41.05",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3018.83,
            "range": "± 15.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 155.57,
            "range": "± 0.82",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 190.85,
            "range": "± 0.64",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 231.78,
            "range": "± 0.59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 174.61,
            "range": "± 1.13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2916.61,
            "range": "± 33.05",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 32803.96,
            "range": "± 66.67",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 345303.5,
            "range": "± 709.12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1757069.59,
            "range": "± 4078.86",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 151.55,
            "range": "± 3.56",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11070.79,
            "range": "± 35.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1055.52,
            "range": "± 3.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 309.05,
            "range": "± 11.42",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 337.36,
            "range": "± 13.18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 380.66,
            "range": "± 17.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 317.2,
            "range": "± 10.54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 461.61,
            "range": "± 5.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 611.74,
            "range": "± 9.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 5.4,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 48.55,
            "range": "± 0.05",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 48.54,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 48.57,
            "range": "± 0.11",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2215.44,
            "range": "± 6.41",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 63492.43,
            "range": "± 117.28",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 238.46,
            "range": "± 1.69",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6437.46,
            "range": "± 25.46",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1110.47,
            "range": "± 4.78",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 31768.93,
            "range": "± 182.13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 125413.7,
            "range": "± 394.08",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 32711.41,
            "range": "± 103.23",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 510629,
            "range": "± 2573.08",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8325.24,
            "range": "± 29.15",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9178139.66,
            "range": "± 386006.15",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9189871.64,
            "range": "± 549278.11",
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
            "name": "compression_overhead_without_compression_1",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 102473.89,
            "range": "± 1269.18",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 56531.68,
            "range": "± 515.92",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4332390192.5,
            "range": "± 49371249.74",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4453492286.3,
            "range": "± 49577208.65",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4576598279.2,
            "range": "± 45876792.5",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4776493079.6,
            "range": "± 32239781.61",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4354190873.1,
            "range": "± 45251727.96",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4463034903.8,
            "range": "± 37921295.81",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4583515933.7,
            "range": "± 42825460.42",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4398213893,
            "range": "± 41340106.32",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4309596030.2,
            "range": "± 13760327.52",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4312156741.2,
            "range": "± 52946611.85",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4311241729,
            "range": "± 25311904.94",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7947373308,
            "range": "± 96951947.81",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4528464397.7,
            "range": "± 44101709.9",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5269076671.5,
            "range": "± 77146521.28",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6167141297.2,
            "range": "± 41580050.87",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 27648.86,
            "range": "± 170.38",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 621964.39,
            "range": "± 803.66",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 13967.99,
            "range": "± 1163.09",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 40837.63,
            "range": "± 2387.56",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 81268.98,
            "range": "± 4582.85",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 128294.98,
            "range": "± 9364.12",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 6221491.83,
            "range": "± 389371.68",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 11418787.18,
            "range": "± 1179261.12",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 30927644.97,
            "range": "± 3358979.58",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 243.14,
            "range": "± 2.73",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2190.42,
            "range": "± 6.23",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1106.96,
            "range": "± 10.94",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6451.8,
            "range": "± 7.86",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 63518.22,
            "range": "± 61.52",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 31769.57,
            "range": "± 45.54",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 18873.71,
            "range": "± 91.71",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 31896.78,
            "range": "± 1959.56",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 40920.67,
            "range": "± 2308.39",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1561.02,
            "range": "± 2.93",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 5.36,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 828.72,
            "range": "± 3.3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 821.67,
            "range": "± 3.51",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 768.71,
            "range": "± 1.43",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 517.53,
            "range": "± 12.9",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 406.58,
            "range": "± 2.96",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6009790.03,
            "range": "± 598999.67",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2986004.55,
            "range": "± 191351.6",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14907.31,
            "range": "± 150.58",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 794.54,
            "range": "± 1.77",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 21390.33,
            "range": "± 25061.95",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 996.72,
            "range": "± 12.04",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27.6,
            "range": "± 0.31",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 22559.82,
            "range": "± 313.44",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10383.22,
            "range": "± 69.24",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 46976.03,
            "range": "± 511.21",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 27.24,
            "range": "± 0.11",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 6027.48,
            "range": "± 31.44",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4111.48,
            "range": "± 40.59",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 11147.88,
            "range": "± 31.33",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3084.98,
            "range": "± 12.32",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 27695.83,
            "range": "± 146.17",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 269174.52,
            "range": "± 980.03",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2586743.58,
            "range": "± 5800.63",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 165202.23,
            "range": "± 433.49",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 13026.9,
            "range": "± 39.51",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 910.79,
            "range": "± 9.73",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 308925.92,
            "range": "± 1664.12",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 25838,
            "range": "± 345.85",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2554.15,
            "range": "± 10.58",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 248.34,
            "range": "± 1.43",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 164761.03,
            "range": "± 483.63",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 165178.59,
            "range": "± 809.6",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 165250.78,
            "range": "± 494.46",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 165142.95,
            "range": "± 495.17",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 12810.47,
            "range": "± 82.41",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 13933.85,
            "range": "± 126.6",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 25613.27,
            "range": "± 98.06",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 93764.04,
            "range": "± 922.36",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 39757.17,
            "range": "± 316.77",
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
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "04744db9e60fbeb0c75c4f64a3a02c41957988af",
          "message": "fix(ci): ingest only fresh Criterion new/ estimates in benchmarks (#1022)\n\nThe conversion step read every estimates.json under the criterion dir and\nstripped /new/, /base/ and /change/ down to a single benchmark name, emitting\nall of them as absolute ns. A scratch Criterion 0.8 bench shows base/ is a\nbyte-identical copy of new/ and change/ holds RELATIVE deltas (e.g. -0.099),\nwhich the old code emitted as \"-1 ns/iter\". CI evidence: run 34879770281\n(232 files -> 222 lines -> 111 entries) and the PR #1005 run\n(233 files -> 223 lines -> 112 entries, with 5 compression_overhead/\nwithout_compression benchmarks silently dropped because floor(0.311 ns) = 0).\n\n- restrict ingestion to -path '*/new/estimates.json'\n- emit 2-decimal floats instead of flooring, so sub-ns means survive; the\n  action's cargo parser accepts them ([0-9,.]+ value/range groups)\n- fail closed on estimates with no positive ns mean (was a silent drop)\n- fail closed when converted rows != fresh file count (provenance)\n- fail closed on duplicate benchmark names after conversion\n- extend scripts/test-benchmark-workflow.sh fixtures (sub-ns + freshness)\n- record LESSON-025 and the ingest rule in github_actions_patterns.md",
          "timestamp": "2026-09-15T12:25:07Z",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/04744db9e60fbeb0c75c4f64a3a02c41957988af"
        },
        "date": 1789478379595,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 47781.07,
            "range": "± 219.71",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18333.85,
            "range": "± 58.08",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 178949.05,
            "range": "± 590.08",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31570.78,
            "range": "± 94.07",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156496.5,
            "range": "± 2115.5",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 43317186.02,
            "range": "± 1083035.53",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 400412380.3,
            "range": "± 8141937.78",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 201175368.13,
            "range": "± 10037197.55",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 399176003.2,
            "range": "± 5234918.06",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 81829411.99,
            "range": "± 1744354.64",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 24993123.53,
            "range": "± 587811.54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183.24,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 350.84,
            "range": "± 1625.59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9.98,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 20.44,
            "range": "± 0.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2218082.58,
            "range": "± 41110.52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 547.58,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3666.44,
            "range": "± 37.74",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 197.28,
            "range": "± 2.29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 235.72,
            "range": "± 1.16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 286.82,
            "range": "± 2.39",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 211.62,
            "range": "± 1.69",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3712.58,
            "range": "± 40.28",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40833.37,
            "range": "± 193.65",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 428205.41,
            "range": "± 2052.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2227819.96,
            "range": "± 15690",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 185.13,
            "range": "± 2.26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12882.7,
            "range": "± 125.66",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1240.32,
            "range": "± 4.23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 426.23,
            "range": "± 36.98",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 466.66,
            "range": "± 35.88",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 502.49,
            "range": "± 40.85",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 422.7,
            "range": "± 22.67",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 553.09,
            "range": "± 7.25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 735.25,
            "range": "± 12.54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.22,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58.19,
            "range": "± 0.08",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2790.28,
            "range": "± 22.54",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74766.7,
            "range": "± 119.12",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330.55,
            "range": "± 0.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7556.28,
            "range": "± 14.72",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1426.91,
            "range": "± 6.88",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37266.82,
            "range": "± 101.96",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161540.49,
            "range": "± 2850.96",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41349.6,
            "range": "± 101.28",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 661746.58,
            "range": "± 2398.67",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10583.91,
            "range": "± 43.43",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9703635.53,
            "range": "± 525449.11",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9635157.35,
            "range": "± 282042.56",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.69,
            "range": "± 0.5",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 150088.66,
            "range": "± 2639.89",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 78869.66,
            "range": "± 1436.04",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4466826520.9,
            "range": "± 49182297.5",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4566958761.9,
            "range": "± 45886225.61",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5142614673.5,
            "range": "± 69590031.5",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4461061525.7,
            "range": "± 49252749.25",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4624145454.4,
            "range": "± 56893624.39",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4769357780.7,
            "range": "± 30515547.63",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4847485071.4,
            "range": "± 51682358.99",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4428872517.6,
            "range": "± 53532535.32",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4509620707,
            "range": "± 57557822.05",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4615055919.5,
            "range": "± 60517694.68",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8161611633.7,
            "range": "± 89468340.26",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4657302610.8,
            "range": "± 109868028.42",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5388783908.5,
            "range": "± 75427191.33",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6294320864.5,
            "range": "± 85711731.21",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34656.69,
            "range": "± 1038.49",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 849791.18,
            "range": "± 2477.08",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 31881.96,
            "range": "± 68655.43",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 87418.63,
            "range": "± 38410.33",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 169957.88,
            "range": "± 67505.84",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 216542.05,
            "range": "± 75023.27",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 363.28,
            "range": "± 1.18",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2780.67,
            "range": "± 16.39",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1439.2,
            "range": "± 9.89",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7546.79,
            "range": "± 17.11",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74469.28,
            "range": "± 171.75",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37145.71,
            "range": "± 112.47",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23546.32,
            "range": "± 177.78",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 58485.6,
            "range": "± 36761.81",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 71729.18,
            "range": "± 24771.38",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2215.03,
            "range": "± 12.78",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.17,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1004.47,
            "range": "± 10.08",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1004.52,
            "range": "± 3.37",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 944.34,
            "range": "± 1.21",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 731.36,
            "range": "± 47.3",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 553.4,
            "range": "± 23.38",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6252113.07,
            "range": "± 540495.89",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3114294.11,
            "range": "± 221106.73",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18743.41,
            "range": "± 257.29",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 915.58,
            "range": "± 5.06",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30.83,
            "range": "± 0.24",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28703.58,
            "range": "± 326.49",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13043.48,
            "range": "± 52.18",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 58874.93,
            "range": "± 665.67",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30.19,
            "range": "± 0.12",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7469.92,
            "range": "± 102.71",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4965.35,
            "range": "± 124.52",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13872.23,
            "range": "± 76.38",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3901.18,
            "range": "± 16.45",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35855.86,
            "range": "± 79.01",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349564.65,
            "range": "± 802.01",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 53338.7,
            "range": "± 573.61",
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
          "id": "19e88d4a9f44e966e6784eb18a98fc2b42330da3",
          "message": "fix(ci): fail benchmark store above the measured same-code noise envelope (#1025)\n\nfail-threshold defaulted to alert-threshold (110%), so Store Benchmark Results\nfailed on every main push and carried no signal. Measured on the fixed\ningestion pipeline:\n\n- commits af36e3c1 vs 80dcda16 (identical ingestion code): median ratio 1.199,\n  max 1.72x, 77% of 120 benchmarks >10%, 33% >25%, 1% >50%\n- one SHA measured twice (04744db9, push vs workflow_dispatch): median 1.212,\n  max 2.28x, 0 benchmarks above 3x\n- pre-fix history (14 stored runs, 1480 comparisons): max 4.00x on a 5ns\n  timer-floor benchmark, 3.82x for means >=1us\n\n- keep alert-threshold 110% (comment + @maintainers cc) for signal\n- set fail-threshold 500%: strict '>' gives >=25% margin over the observed\n  envelope while still failing closed on catastrophic regressions\n- guard fixtures reject a missing or <200% fail-threshold\n- LESSON-025 corrected: it claimed a threshold the workflow did not have, and\n  its numbers now come from the post-fix pipeline",
          "timestamp": "2026-09-15T18:00:13+02:00",
          "tree_id": "1929b450eb45055cebb2200b133878bb23d2d612",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/19e88d4a9f44e966e6784eb18a98fc2b42330da3"
        },
        "date": 1789491230020,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 64029.13,
            "range": "± 313.12",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 23341.49,
            "range": "± 283.74",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 243757.09,
            "range": "± 564.38",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 41361.14,
            "range": "± 344.84",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 204656.26,
            "range": "± 2966.67",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 65178055.47,
            "range": "± 3514191.51",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 535008583.6,
            "range": "± 48149960.5",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 256898712.55,
            "range": "± 26935280.52",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 510004598.9,
            "range": "± 16677250.53",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 115641197.88,
            "range": "± 10275719.65",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 37938843.92,
            "range": "± 4254052.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 179.37,
            "range": "± 0.68",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 339.04,
            "range": "± 1468.88",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 31.95,
            "range": "± 0.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 37.65,
            "range": "± 0.12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2607105.43,
            "range": "± 28458.87",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 598.13,
            "range": "± 50.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3663.13,
            "range": "± 32.95",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 195.76,
            "range": "± 0.75",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 307.5,
            "range": "± 0.82",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 434.94,
            "range": "± 1.53",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 244.69,
            "range": "± 0.52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 4468.66,
            "range": "± 9.54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 49364.41,
            "range": "± 317.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 509253.38,
            "range": "± 2091.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2535136.26,
            "range": "± 20772.42",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 198.78,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11961.41,
            "range": "± 141.4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1132.57,
            "range": "± 6.63",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 406.67,
            "range": "± 24.97",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 511.88,
            "range": "± 34.49",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 635.38,
            "range": "± 57.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 452.33,
            "range": "± 24.46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 560,
            "range": "± 9.03",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 732.55,
            "range": "± 11.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 20.85,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 48.24,
            "range": "± 0.13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2766.42,
            "range": "± 15.64",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 78338.1,
            "range": "± 356.35",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 284.68,
            "range": "± 2.76",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7912.93,
            "range": "± 19.07",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1449.16,
            "range": "± 11.17",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 39202.57,
            "range": "± 136.58",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 212067.92,
            "range": "± 2794.05",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 53141.29,
            "range": "± 673.34",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 852174.4,
            "range": "± 10514.25",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 12817.95,
            "range": "± 154.12",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 13896078.54,
            "range": "± 1007124.56",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 16540767.09,
            "range": "± 2802283.51",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.39,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6.39,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6.4,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6.4,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 116938.42,
            "range": "± 2014.24",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 74758.72,
            "range": "± 955.77",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 6761533373.1,
            "range": "± 322364555.07",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 6345676092.1,
            "range": "± 336791882.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 6725477975.4,
            "range": "± 918488189.82",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 6400168198.6,
            "range": "± 546475740.37",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5361089170,
            "range": "± 253350283.64",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 5765651725.9,
            "range": "± 370180544.45",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5660012658.7,
            "range": "± 147875411.53",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 10731683276.6,
            "range": "± 577372579.76",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5972650524.7,
            "range": "± 308024451.7",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 7024321828.7,
            "range": "± 563464183.55",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 7833831853.4,
            "range": "± 305786286.37",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 23874.33,
            "range": "± 287.52",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 669201.89,
            "range": "± 12753.79",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 27406.81,
            "range": "± 7973.15",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 82281.1,
            "range": "± 20177.11",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 175262.74,
            "range": "± 47731.17",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 223993.84,
            "range": "± 49604.36",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 323.28,
            "range": "± 1.32",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2764.31,
            "range": "± 8.56",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1439.56,
            "range": "± 4.49",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7907.41,
            "range": "± 18.94",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 78267.39,
            "range": "± 335.95",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 39205.41,
            "range": "± 83.03",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23112.95,
            "range": "± 101.68",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 81973.91,
            "range": "± 27010.22",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 79565.54,
            "range": "± 18776.62",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 3278.71,
            "range": "± 7.14",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 20.85,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1301.24,
            "range": "± 5.3",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1298.01,
            "range": "± 4.61",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1172.26,
            "range": "± 10.08",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 788.04,
            "range": "± 54.47",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 586.48,
            "range": "± 19.12",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 11116570.47,
            "range": "± 2663836.85",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 5052756.5,
            "range": "± 766142.92",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14420.2,
            "range": "± 402.6",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1178.74,
            "range": "± 5.08",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 31.85,
            "range": "± 0.19",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 27324.58,
            "range": "± 302.28",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12447.21,
            "range": "± 120.1",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 56495.34,
            "range": "± 522.12",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 77.5,
            "range": "± 0.11",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7910.91,
            "range": "± 106.04",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5293.88,
            "range": "± 46.01",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14548.21,
            "range": "± 158.92",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3421.21,
            "range": "± 28.09",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 29746.12,
            "range": "± 226.74",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 296737.22,
            "range": "± 2723.86",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 61629.21,
            "range": "± 515.53",
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
          "id": "452eb21aba14f6305794af4e21a9cdda2171f535",
          "message": "refactor(loc): restore the 500 LOC invariant that blocked the v0.1.41 release (#1026)\n\n* refactor(loc): keep six source files under the 500 LOC ceiling\n\nThe package-private LOC gate in scripts/quality-gates.sh\n(`run_source_file_size_gate`) had been failing on main: six source files\ngrew past the 500 LOC core invariant since the v0.1.40 tag, which blocked\n`release-manager ship` even though main CI stayed green (the CI Quality\nGates job only runs the security audit and coverage steps).\n\nEvery move is behaviour-preserving; no public path changes:\n\n- `memory-core/src/patterns/similarity.rs` 533 -> 292: inline test module\n  to `similarity/tests.rs`\n- `memory-mcp/src/mcp/tools/checkpoint/tool.rs` 713 -> 466: inline test\n  module to `tool/tests.rs`\n- `memory-cli/src/commands/episode/core/checkpoint.rs` 554 -> 383: inline\n  test module to `checkpoint/tests.rs`\n- `memory-cli/src/commands/episode/core/types.rs` 503 -> 304: command\n  payload enums and DTOs to `types/models.rs`, re-exported from `types`\n- `memory-core/src/memory/checkpoint/compact.rs` 514 -> 450: byte-budget\n  and truncation helpers to `compact/budget.rs` as `pub(super)`\n- `memory-core/src/storage/backend.rs` 502 -> 490: drop three `# Arguments`\n  doc blocks that only restated the signature (`# Errors` kept)\n\nVerified: cargo fmt --check clean, clippy --workspace --tests with\n-D warnings clean, 3157/3158 tests pass for the three touched crates (the\nremaining test is a pre-existing 80s subprocess integration test that\ntimes out only under parallel load).\n\n* fix(scripts): make the loc sensor scan real workspace paths\n\n`scripts/check-loc.sh` (do-harness `loc` sensor and pre-commit hook)\nwalked `$ROOT/src` and `$ROOT/crates`, neither of which exists in this\nworkspace, so `find` matched nothing and the sensor printed\n`check-loc OK` on every commit. That vacuous pass is why six files drifted\npast the 500 LOC invariant unnoticed until `release-manager ship` hit the\nblocking gate in `scripts/quality-gates.sh`.\n\n- Select files with `git ls-files '*.rs'` plus untracked ones, excluding\n  `target/` and `benches/`, and apply the same test-file carve-out as the\n  gate so sensor and gate agree.\n- Report the >=450 decomposition threshold as a compact summary instead of\n  listing ~50 files on every commit.\n- Record LESSON-026: a sensor that scans missing paths reports success\n  forever, so assert the scan found files before reporting OK.\n\nVerified: check-loc.sh exits 0 with 0 blocking violations and 60\nnon-blocking oversized test files; the gate-equivalent scan over 1117\nRust files reports 0 blocking violations.",
          "timestamp": "2026-09-17T16:26:12+02:00",
          "tree_id": "c283cfbddabe38b69aa08b0525a4b3d831572563",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/452eb21aba14f6305794af4e21a9cdda2171f535"
        },
        "date": 1789658336837,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 50502.78,
            "range": "± 458.96",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18202.1,
            "range": "± 358.03",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 190491.08,
            "range": "± 523.43",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 32436.85,
            "range": "± 374.46",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 157778,
            "range": "± 1247.68",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 57048334.64,
            "range": "± 39101097.84",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 372896946.2,
            "range": "± 6941481.43",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 182751592.67,
            "range": "± 2414694.64",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 376067168.3,
            "range": "± 7586030.21",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 70894443.37,
            "range": "± 2309872.75",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 18965874.71,
            "range": "± 361749.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 197.85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 319.57,
            "range": "± 1316.97",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 11.28,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 21.14,
            "range": "± 0.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2004027.58,
            "range": "± 16190.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 462.81,
            "range": "± 38.23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3875.19,
            "range": "± 19.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 200.68,
            "range": "± 1.63",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 246.22,
            "range": "± 1.39",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 299.24,
            "range": "± 0.91",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 224.78,
            "range": "± 1.5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3749.1,
            "range": "± 19.19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 42273.62,
            "range": "± 302.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 443751.99,
            "range": "± 1923.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2242813.29,
            "range": "± 8002.21",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 189.89,
            "range": "± 1.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 14306.4,
            "range": "± 84.22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1356.04,
            "range": "± 4.81",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 390.82,
            "range": "± 11.51",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 431.73,
            "range": "± 21.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 483.48,
            "range": "± 22.79",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 401.01,
            "range": "± 11.56",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 594.17,
            "range": "± 8.5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 790.03,
            "range": "± 14.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.95,
            "range": "± 0.05",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 62.6,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2847.96,
            "range": "± 12.82",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 81779.94,
            "range": "± 234.45",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 308.07,
            "range": "± 2.49",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 8292.38,
            "range": "± 15.93",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1418.04,
            "range": "± 3.89",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 40929.73,
            "range": "± 163.53",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161540.3,
            "range": "± 583.75",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41623.15,
            "range": "± 296.21",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 658090.99,
            "range": "± 2295.4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10508.12,
            "range": "± 52.59",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 7176690.09,
            "range": "± 131081.14",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 7262798.45,
            "range": "± 160805.11",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 7.74,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 7.73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 7.73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 7.73,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 7.73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 131580.87,
            "range": "± 2130.74",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 73761.89,
            "range": "± 754.78",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4329578949.5,
            "range": "± 41989570.46",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4466572387.1,
            "range": "± 32216888.57",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 5283391611.2,
            "range": "± 2155704256.45",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4951707330.8,
            "range": "± 44262221.45",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4360157060,
            "range": "± 57738677.12",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4500779286.5,
            "range": "± 41881796.15",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4608139560.5,
            "range": "± 59214149.13",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4719559734.5,
            "range": "± 44375575.72",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4348573431.1,
            "range": "± 46549450",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4389158700.4,
            "range": "± 29976464.52",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4523698960.1,
            "range": "± 43499991.93",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8025194683.4,
            "range": "± 80286526.04",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4541035001.1,
            "range": "± 41744487.86",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5259276783.3,
            "range": "± 51038912.35",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6106994771.9,
            "range": "± 74038897.05",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 35115.36,
            "range": "± 146.08",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 887144.36,
            "range": "± 3574.28",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 23243.59,
            "range": "± 59403.58",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 50862.55,
            "range": "± 2274.6",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 102517.07,
            "range": "± 4297.26",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 152741.45,
            "range": "± 5844.06",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 317.78,
            "range": "± 0.96",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2827.28,
            "range": "± 15.58",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1422.5,
            "range": "± 8.25",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 8318.25,
            "range": "± 12.4",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 81601,
            "range": "± 138.77",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 40965.46,
            "range": "± 93.15",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 24113.19,
            "range": "± 133.8",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 42918.81,
            "range": "± 31459.4",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 50328.8,
            "range": "± 2242.56",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2005.52,
            "range": "± 4.4",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1046.04,
            "range": "± 4.67",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1042.35,
            "range": "± 4.41",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 994.19,
            "range": "± 5.96",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 658.01,
            "range": "± 12.85",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 516.34,
            "range": "± 5.62",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 4548785.9,
            "range": "± 554140.7",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2268752.41,
            "range": "± 33212.32",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 19210.49,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 972.69,
            "range": "± 1.7",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 34.82,
            "range": "± 0.13",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29385.97,
            "range": "± 500.31",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13484.13,
            "range": "± 104.72",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 61983.21,
            "range": "± 584.96",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 35.08,
            "range": "± 0.13",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7888.22,
            "range": "± 96.72",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5265.63,
            "range": "± 79.97",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14583.37,
            "range": "± 259.77",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 4362.54,
            "range": "± 30.04",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 37878.5,
            "range": "± 100.66",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 367924.17,
            "range": "± 2126.31",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 16424.28,
            "range": "± 63.58",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1168.72,
            "range": "± 7.28",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3061.39,
            "range": "± 14.06",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 232.16,
            "range": "± 2.49",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50816.28,
            "range": "± 1233.17",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "Dominik Oswald",
            "username": "d-oit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "471ab7d502fccfc98fb66823bd232321ca93c1d5",
          "message": "ci(mutants): bound shards so the workflow cannot end cancelled (#1028)\n\n`release-manager.sh ci-check` requires every completed run on `origin/main`\nHEAD to conclude success/skipped/neutral, and it counts `cancelled` as\nfailed. Each `Mutation Testing` shard carries `timeout-minutes: 60` but\nneeds longer, so GitHub kills the shard, the run concludes `cancelled`, and\nevery main commit touching `memory-core/src/{reward,retrieval,retry,\npatterns}/**` blocks the release path. All 12 runs in the month before this\nchange concluded `cancelled` for the same reason.\n\nThe shard step now wraps `cargo mutants` in\n`timeout --signal=TERM --kill-after=60s \"$MUTANTS_SHARD_BUDGET_SECONDS\"`\n(default 3000s), so it exits before the job timeout and the run concludes\n`success`. rc 124/137 reports truncation via a notice; any other non-zero rc\nreports a warning. The job stays `continue-on-error: true` because Phase 1\nof issue #747 is informational and partial artifacts remain useful.\n\nVerified: YAML parses, `actionlint` clean, and the step's failure path is\nexercised by the `|| rc=$?` capture rather than `set -e` aborting the step.\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-17T18:29:42+02:00",
          "tree_id": "cfb6574609e71ee3e760a60bc09fc1a9aec7ca4a",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/471ab7d502fccfc98fb66823bd232321ca93c1d5"
        },
        "date": 1789665696356,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48008.15,
            "range": "± 143.55",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18078.74,
            "range": "± 86.26",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 177933.16,
            "range": "± 821.31",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31800.85,
            "range": "± 152.33",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 155863.97,
            "range": "± 603.53",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 46839962.88,
            "range": "± 1856286.68",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 420330763.4,
            "range": "± 8803947.76",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 215567458,
            "range": "± 10279540.2",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 427849990.65,
            "range": "± 25534606.91",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 90517503.55,
            "range": "± 5719244.46",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 27515932.68,
            "range": "± 492994.06",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183.54,
            "range": "± 1.23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 330.08,
            "range": "± 1475.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9.97,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 20.4,
            "range": "± 0.37",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2116544.07,
            "range": "± 31566.66",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 463.77,
            "range": "± 50.18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3675.04,
            "range": "± 37.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 197.2,
            "range": "± 1.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 235.47,
            "range": "± 1.38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 285.69,
            "range": "± 1.16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 211.46,
            "range": "± 0.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3741.24,
            "range": "± 73.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 41173.02,
            "range": "± 281.99",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 434409.14,
            "range": "± 1756.91",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2208964.31,
            "range": "± 8368.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 185.32,
            "range": "± 1.81",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12866.95,
            "range": "± 45.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1239.28,
            "range": "± 3.4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 394.16,
            "range": "± 12.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 429.03,
            "range": "± 13.46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 476,
            "range": "± 21.34",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 413.64,
            "range": "± 14.97",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 552.91,
            "range": "± 7.11",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 736.97,
            "range": "± 12.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.22,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58.2,
            "range": "± 0.11",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2780.59,
            "range": "± 16.73",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74330.03,
            "range": "± 245.28",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330.24,
            "range": "± 1.52",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7554.99,
            "range": "± 19.33",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1419.91,
            "range": "± 5.72",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37235.33,
            "range": "± 98.67",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 160822.99,
            "range": "± 872.49",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 40959.15,
            "range": "± 111.83",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 658502.35,
            "range": "± 8789.98",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10391.18,
            "range": "± 36.86",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 11058385.08,
            "range": "± 644480.31",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 11257727.5,
            "range": "± 544722.69",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 145670.42,
            "range": "± 1768.38",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 78643.1,
            "range": "± 772.52",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4542582927.3,
            "range": "± 44800486.71",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4825113287.9,
            "range": "± 139287578.52",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5272141480.1,
            "range": "± 36274943.17",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4537425822.2,
            "range": "± 59402331.99",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4729172096.2,
            "range": "± 93718785.97",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4898934680.3,
            "range": "± 78832351.55",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4904095679.5,
            "range": "± 93732633.51",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4555010063,
            "range": "± 45417745.1",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4607506827.4,
            "range": "± 38521904.98",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4736387949.5,
            "range": "± 90191587.09",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8261390018,
            "range": "± 104481737.87",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4746940777.9,
            "range": "± 61063773.9",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5455042092.3,
            "range": "± 103591615.41",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6369292652.1,
            "range": "± 64138751.39",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34205.18,
            "range": "± 413.88",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 853944.28,
            "range": "± 2198.56",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 18064.46,
            "range": "± 4462.13",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 49224.78,
            "range": "± 5761.75",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 104117.08,
            "range": "± 10885.44",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 156654.14,
            "range": "± 24070.02",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 365.87,
            "range": "± 4.45",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2777.07,
            "range": "± 6.2",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1427.3,
            "range": "± 4.04",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7545.09,
            "range": "± 33.25",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74521.23,
            "range": "± 125.04",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37090.12,
            "range": "± 53.33",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23312.69,
            "range": "± 85.19",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 42884.09,
            "range": "± 6569.1",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 49394.29,
            "range": "± 3837.18",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2221.8,
            "range": "± 39.73",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.07,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1009.36,
            "range": "± 2.79",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 997.65,
            "range": "± 4.59",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 957.79,
            "range": "± 6.36",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 667.85,
            "range": "± 11.96",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 526.03,
            "range": "± 11.23",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 7005826.66,
            "range": "± 744141.45",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3435791.07,
            "range": "± 299427.95",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18702.66,
            "range": "± 98.08",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 913.19,
            "range": "± 1.95",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30.83,
            "range": "± 0.3",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28956.43,
            "range": "± 188.67",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12956.77,
            "range": "± 227.59",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 59104.17,
            "range": "± 709.92",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30.12,
            "range": "± 0.05",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7633.72,
            "range": "± 111.3",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5165.75,
            "range": "± 57.4",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13777.92,
            "range": "± 107.65",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3840.95,
            "range": "± 21.05",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35590.08,
            "range": "± 124.32",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349091.54,
            "range": "± 1452.38",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 187943.87,
            "range": "± 2523.5",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14290.99,
            "range": "± 61.52",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1011.5,
            "range": "± 8.71",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2894.99,
            "range": "± 11.07",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 374.29,
            "range": "± 6.45",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 52575.69,
            "range": "± 1843.85",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "Dominik Oswald",
            "username": "d-oit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b0d2d5c54b86020a087bd255e10f06bdcf23252",
          "message": "test(config): size local nextest caps to measured CLI-wave runtime (#1029)\n\n* test(config): size local nextest caps to measured CLI-wave runtime\n\n`cargo nextest run --all` — the local gate `release-manager ship` runs —\nkilled three CLI-subprocess tests mid-run while they were still passing:\n`e2e-tests::cli_workflows::test_bulk_operations` (134.5s measured) and\n`do-memory-cli::relationship_command_tests::test_relationship_full_cycle`\n(179.0s) at the 120s default cap, and `test_pattern_discovery` (400.5s) at\nits 270s cap. Each spawns the debug CLI binary once per command, so runtime\ntracks machine speed; running them idle and single-threaded did not help.\n\n`profile.default` gains whole-binary overrides with\n`period = \"240s\", terminate-after = 2` (kill at 480s, above the slowest\nmeasured test) for `binary(cli_workflows)` and\n`binary(relationship_command_tests)`. The per-test default override for\n`test_pattern_discovery` is removed: when several overrides match, the\nbinary-level one wins, so two entries with different caps for the same test\nmade the effective limit unpredictable. `profile.ci` is untouched — it\nretries twice and passes on its runners.\n\nMeasured uncapped with\n`cargo test -p <crate> --test <binary> <test> -- --exact` (handy detail now\nrecorded as LESSON-028).\n\nVerified: `cargo nextest run --all` 3955/3955 passed, exit 0, slowest test\n374.7s of the 480s cap.\n\n* chore(harness): record steering-loop events for today's three sensor fixes\n\nAGENTS.md's steering loop requires a structured event log whenever a sensor\nfires and is resolved. Three fired today while preparing v0.1.41 and none was\nrecorded yet:\n\n- `loc` (maintainability, 6 violations): the sensor walked `src/` and\n  `crates/`, which do not exist here, so it passed vacuously while six files\n  drifted past the 500 LOC invariant; fixed in PR #1026.\n- release-manager ci-check vs `mutants.yml` (12 cancelled runs): shards were\n  killed by their own 60-minute job timeout, so the run concluded\n  `cancelled`, which the release gate counts as failed; fixed in PR #1028.\n- `nextest::slow-timeout` (3 violations): the default profile capped\n  CLI-subprocess tests below their measured runtime; fixed in PR #1029.\n\nEach file records sensor, category, violation count, root cause, the guide or\nconfig updated, and the resolution, so the audit trail is searchable.\n\n---------\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-17T23:34:29+02:00",
          "tree_id": "fb08aeae92827de1c392a7dd0b0bd5b511010893",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/2b0d2d5c54b86020a087bd255e10f06bdcf23252"
        },
        "date": 1789684309243,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 62411.36,
            "range": "± 576.2",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 22789.81,
            "range": "± 235.59",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 237645.86,
            "range": "± 1974.51",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 40301.93,
            "range": "± 329.03",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 200758.32,
            "range": "± 1758.23",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 59691954.78,
            "range": "± 5931755.42",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 484955537.2,
            "range": "± 54480538.04",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 255440195.9,
            "range": "± 24113191.55",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 472149338.05,
            "range": "± 20017392.01",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 97838227.9,
            "range": "± 6140514.37",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 36473569.5,
            "range": "± 2222794.3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 176.37,
            "range": "± 1.95",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 328.31,
            "range": "± 1389.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 31.45,
            "range": "± 0.31",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 37.04,
            "range": "± 0.35",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2451516.39,
            "range": "± 28063.77",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 515.48,
            "range": "± 51.75",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3548,
            "range": "± 42.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 191.25,
            "range": "± 1.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 304.05,
            "range": "± 20.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 422.97,
            "range": "± 3.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 239.55,
            "range": "± 1.86",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 4410.99,
            "range": "± 36.24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 47929.91,
            "range": "± 348.92",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 498142.26,
            "range": "± 3144.64",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2485029.29,
            "range": "± 14993.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 194.53,
            "range": "± 4.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11714.83,
            "range": "± 148.27",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1116.25,
            "range": "± 16.83",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 381.85,
            "range": "± 12.92",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 485.85,
            "range": "± 25.23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 597.08,
            "range": "± 44.4",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 439.85,
            "range": "± 45.21",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 554.59,
            "range": "± 7.45",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 719.04,
            "range": "± 11.66",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 20.49,
            "range": "± 0.2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 47.41,
            "range": "± 0.55",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2742.09,
            "range": "± 21.11",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 77818.98,
            "range": "± 819.26",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 281.56,
            "range": "± 3.02",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7861.45,
            "range": "± 52.81",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1429.5,
            "range": "± 13.91",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 38861.98,
            "range": "± 286.03",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 206696.11,
            "range": "± 1998.29",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 51235.62,
            "range": "± 435.77",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 828893.43,
            "range": "± 7219.19",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 12496.5,
            "range": "± 133.76",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 12708229.29,
            "range": "± 736429.26",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 12895066.16,
            "range": "± 851872.58",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.36,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6.38,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6.37,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6.37,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.37,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.34,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 105635.88,
            "range": "± 1471.24",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 69823.36,
            "range": "± 920.27",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 6134365356.3,
            "range": "± 280598109.03",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 5275743217.5,
            "range": "± 312321255.31",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 5295290146.1,
            "range": "± 185334756.96",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5799948188.8,
            "range": "± 198625398.12",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 5574686683.6,
            "range": "± 269244258.88",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5085421615.4,
            "range": "± 188762173.89",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 5398831862.4,
            "range": "± 380642283.14",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5403955041.3,
            "range": "± 248123314.15",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 9916986702.3,
            "range": "± 389284247.58",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5222219282.9,
            "range": "± 181868249.82",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 6375147807.4,
            "range": "± 330537317.87",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 7355881894.6,
            "range": "± 258337635.63",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 23723.14,
            "range": "± 163.69",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 642075.17,
            "range": "± 13523.84",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 19974.64,
            "range": "± 3367.47",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 58619.19,
            "range": "± 4868.7",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 116845.96,
            "range": "± 8134.09",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 171256.47,
            "range": "± 8872.83",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 316.05,
            "range": "± 2.37",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2701.49,
            "range": "± 24.12",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1414.62,
            "range": "± 8.88",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7737.98,
            "range": "± 58.7",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 76844.5,
            "range": "± 371.51",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 38458.28,
            "range": "± 220.06",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 22448.18,
            "range": "± 317.4",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 55824.22,
            "range": "± 7788.34",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 59303.76,
            "range": "± 4619.99",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 3213.85,
            "range": "± 44.59",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 20.29,
            "range": "± 0.46",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1256.79,
            "range": "± 3.52",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1265.27,
            "range": "± 2.64",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1142.64,
            "range": "± 3.49",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 694.76,
            "range": "± 22.96",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 532.66,
            "range": "± 10.37",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 8384781.67,
            "range": "± 1504807.06",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 4448995.52,
            "range": "± 1265087.68",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14045.89,
            "range": "± 449.21",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1146.2,
            "range": "± 2.56",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 31.11,
            "range": "± 0.25",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 26888.5,
            "range": "± 771.57",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12211.16,
            "range": "± 147.46",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 55002.43,
            "range": "± 910.76",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 73.55,
            "range": "± 0.65",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7805.44,
            "range": "± 76.89",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5218.05,
            "range": "± 75.99",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14513.44,
            "range": "± 148.12",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3468.79,
            "range": "± 290.58",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 28887.16,
            "range": "± 218.35",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 283839.46,
            "range": "± 2812.89",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 59367.48,
            "range": "± 443.2",
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
          "id": "ba95bc88c1c98aa480c84af333102c3645b5e713",
          "message": "fix(framework): cap limit parameter in DomainIndex::get_recent_episodes (#1033)\n\n* fix(framework): cap limit parameter in DomainIndex::get_recent_episodes\n\n- Cap limit using crate::storage::MAX_QUERY_LIMIT in get_recent_episodes\n- Add test_get_recent_episodes_unbounded_limit to verify bound enforcement\n- Prevents unbounded result set memory allocation on DomainIndex queries\n\nCo-authored-by: d-o-hub <242170972+d-o-hub@users.noreply.github.com>\n\n* fix(framework): cap limit parameter in DomainIndex::get_recent_episodes\n\n- Cap limit using crate::storage::MAX_QUERY_LIMIT in get_recent_episodes\n- Add test_get_recent_episodes_unbounded_limit to verify bound enforcement\n- Prevents unbounded result set memory allocation on DomainIndex queries\n\nCo-authored-by: d-o-hub <242170972+d-o-hub@users.noreply.github.com>\n\n---------\n\nCo-authored-by: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-17T23:25:00Z",
          "tree_id": "24640656af0b6435ed8d587b09ae7d95519fc32f",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/ba95bc88c1c98aa480c84af333102c3645b5e713"
        },
        "date": 1789690602116,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 50173.73,
            "range": "± 365.67",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18436.13,
            "range": "± 847.39",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 198028.18,
            "range": "± 718.52",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 32697.28,
            "range": "± 140.67",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156791.04,
            "range": "± 1548.32",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 34831288.01,
            "range": "± 1823894.8",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 378513411.75,
            "range": "± 9613424.79",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 193270454.83,
            "range": "± 13592997.92",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 378838625,
            "range": "± 11529315.45",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 71232082.59,
            "range": "± 2456799.25",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 20128612.09,
            "range": "± 991559.09",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 197.8,
            "range": "± 1.07",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 326.28,
            "range": "± 1376.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 11.27,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 21.17,
            "range": "± 0.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2040000.54,
            "range": "± 27981.13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 506.63,
            "range": "± 51.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3838.08,
            "range": "± 30.81",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 200.69,
            "range": "± 3.09",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 245.89,
            "range": "± 1.67",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 299.2,
            "range": "± 1.22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 225.19,
            "range": "± 1.35",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3720.5,
            "range": "± 48.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 41956.74,
            "range": "± 663.18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 441077.13,
            "range": "± 3105.45",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2256475.97,
            "range": "± 7189.38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 190.42,
            "range": "± 1.64",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 14357.79,
            "range": "± 303.41",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1353.88,
            "range": "± 5.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 395.67,
            "range": "± 17.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 433.83,
            "range": "± 18.7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 487.67,
            "range": "± 22.59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 407.89,
            "range": "± 15.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 595.54,
            "range": "± 7.61",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 793.59,
            "range": "± 15.13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.96,
            "range": "± 0.05",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 62.6,
            "range": "± 0.06",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2854.84,
            "range": "± 63.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 82156.42,
            "range": "± 1002.67",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 309.8,
            "range": "± 13.18",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 8321.63,
            "range": "± 29.75",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1436.86,
            "range": "± 60.33",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 40964.06,
            "range": "± 83.65",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 163545.69,
            "range": "± 1271.09",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 42027.24,
            "range": "± 220.18",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 671969.4,
            "range": "± 37912.84",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10465.96,
            "range": "± 68.6",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 7618556.1,
            "range": "± 253998.5",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 7344620.45,
            "range": "± 153886.11",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 7.79,
            "range": "± 0.14",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 7.73,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 7.73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 7.74,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 7.77,
            "range": "± 0.11",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 132280.74,
            "range": "± 1820.7",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 73003.21,
            "range": "± 633.36",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4344304171.7,
            "range": "± 67679516.91",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4553365906.4,
            "range": "± 79386142.89",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4680850873.3,
            "range": "± 75233221.73",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5000596078.2,
            "range": "± 47811617.2",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4364645282.3,
            "range": "± 40836825.55",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4499236859.7,
            "range": "± 56314534",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4669246620.5,
            "range": "± 42652741.06",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4759667408,
            "range": "± 57203792.18",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4337378714,
            "range": "± 51673678.15",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4404619869.2,
            "range": "± 52630324.82",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4510708074.4,
            "range": "± 40717823.98",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7973000300.8,
            "range": "± 79834888.31",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4627356193.4,
            "range": "± 71957861",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5287112164.7,
            "range": "± 77880673.86",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6184769075,
            "range": "± 90302433.68",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 35651.57,
            "range": "± 262.08",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 864133.33,
            "range": "± 21993.86",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 19587.33,
            "range": "± 3051.66",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 54649.4,
            "range": "± 7423.82",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 124036.01,
            "range": "± 113731.77",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 157139.03,
            "range": "± 8682.59",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 333.75,
            "range": "± 1.43",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2821.82,
            "range": "± 9.08",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1433.75,
            "range": "± 3.21",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 8326.1,
            "range": "± 22.64",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 82001.73,
            "range": "± 152.03",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 40950.51,
            "range": "± 48.04",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 24172.35,
            "range": "± 392.24",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 74464.45,
            "range": "± 331992.05",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 54766.03,
            "range": "± 5715.36",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2013.28,
            "range": "± 5.54",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1039.73,
            "range": "± 5.93",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1022.34,
            "range": "± 3.06",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 979.63,
            "range": "± 2.57",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 660.9,
            "range": "± 12.65",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 517.87,
            "range": "± 4.32",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 4948601.79,
            "range": "± 349533.02",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2558635.61,
            "range": "± 496757.29",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 19182.56,
            "range": "± 143.85",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 972.99,
            "range": "± 3.24",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 34.83,
            "range": "± 0.14",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 30101.45,
            "range": "± 379.73",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13546.4,
            "range": "± 41.55",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 61231.59,
            "range": "± 732.8",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 35.15,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7950.06,
            "range": "± 143.44",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5351.29,
            "range": "± 85.26",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14705.63,
            "range": "± 180.45",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 4354.13,
            "range": "± 16.92",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 37626.33,
            "range": "± 584.55",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 369200.34,
            "range": "± 1926.94",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 16408.48,
            "range": "± 48.33",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1227.88,
            "range": "± 45.86",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 235.45,
            "range": "± 2.27",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 50240.88,
            "range": "± 379.87",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "6849456+d-oit@users.noreply.github.com",
            "name": "Dominik Oswald",
            "username": "d-oit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "37f9cafa060cf51fe9e744682df5acbf19ec55c4",
          "message": "test(config): give the default profile one retry and a 240s base cap (#1035)\n\n* test(config): give the default profile one retry and a 240s base cap\n\nPer-test overrides fixed the three CLI-wave tests but not the class: on the\nnext full run a different test\n(`do-memory-cli config::types::simple_config_tests::test_simple_config_with_cloud_platform`,\n37s in earlier runs) was starved past the 120s base cap while the CLI waves\nheld all four cores for ~6 minutes.\n\n`profile.default` — the only profile that gates local releases and the only\none with no retries and the tightest cap — now uses `retries = 1` and\n`slow-timeout = { period = \"120s\", terminate-after = 2 }` (240s). This matches\nthe precedent in `ci` (2 retries) and `nightly` (1) and cannot mask a\ndeterministic failure: retries only absorb transient starvation, and the retry\nruns serially after the parallel pass.\n\nVerified: `cargo nextest run --all` 3956/3956 passed, exit 0, 589s, no test\nneeded its retry to be reported as flaky.\n\n* docs(lessons): record why the default profile needed retries, not just caps\n\nLESSON-028 now carries the second iteration: per-test overrides fixed three\nCLI-wave tests but not the class, and the next full run starved a 37s unit\ntest past the base cap. Records the reasoning for `retries = 1` (cannot mask a\ndeterministic failure; the retry runs serially after the parallel pass) and\nthe final green result.\n\n---------\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-20T12:10:50+02:00",
          "tree_id": "30f063fd0e739da84a982be3ff8aa9b7f9dc5e71",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/37f9cafa060cf51fe9e744682df5acbf19ec55c4"
        },
        "date": 1789902085258,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 40905.21,
            "range": "± 2666.82",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14403.34,
            "range": "± 384.12",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 165236,
            "range": "± 13471.32",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 26318.23,
            "range": "± 1833.64",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 134168.14,
            "range": "± 5812.2",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 39515227.59,
            "range": "± 1785316.37",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 374351547.4,
            "range": "± 36118931.09",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 183776373.8,
            "range": "± 4560295.23",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 368826237,
            "range": "± 36842046.94",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 75238585.28,
            "range": "± 12128351.74",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 28215423.64,
            "range": "± 16601859",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 122.12,
            "range": "± 1.32",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 200.63,
            "range": "± 713.65",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 19.6,
            "range": "± 0.19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 24.73,
            "range": "± 0.87",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1221668.03,
            "range": "± 59858.32",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 488.49,
            "range": "± 54.79",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2584.32,
            "range": "± 99.1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 140.12,
            "range": "± 3.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 221.94,
            "range": "± 12.12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 322.02,
            "range": "± 4.47",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 178.98,
            "range": "± 8.34",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2984.36,
            "range": "± 90.12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 32833.29,
            "range": "± 700.81",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 346518.46,
            "range": "± 9302.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1748983.52,
            "range": "± 74370.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 141.18,
            "range": "± 1.78",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 8660.64,
            "range": "± 456.12",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 835.4,
            "range": "± 47.51",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 261.55,
            "range": "± 18.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 343.69,
            "range": "± 19.46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 433.87,
            "range": "± 35.92",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 300.69,
            "range": "± 17.86",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 396.45,
            "range": "± 18.38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 509.07,
            "range": "± 21.73",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 10.25,
            "range": "± 0.24",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 42.44,
            "range": "± 0.82",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 43.27,
            "range": "± 3.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 41.97,
            "range": "± 0.73",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_5000",
            "value": 42.43,
            "range": "± 2.11",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 1595.74,
            "range": "± 44.69",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 56069.44,
            "range": "± 1634.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 178.75,
            "range": "± 2.52",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 5659.11,
            "range": "± 121.38",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 794.09,
            "range": "± 22.59",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 27957.98,
            "range": "± 327.52",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 134455.75,
            "range": "± 2203.89",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 34840.65,
            "range": "± 919.71",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 557032.97,
            "range": "± 28835.57",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8459.04,
            "range": "± 89.26",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 11925231.57,
            "range": "± 2679261.06",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 12578237.63,
            "range": "± 6382293.85",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 3.79,
            "range": "± 0.06",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 3.77,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 3.77,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 3.75,
            "range": "± 0.07",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 3.77,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.23,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.24,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 87476.47,
            "range": "± 3071.98",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 46933.17,
            "range": "± 1767",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_16_read_latest@16",
            "value": 4751463953.9,
            "range": "± 95327604.05",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4325823415.2,
            "range": "± 95003248.14",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4422821929.7,
            "range": "± 46801268.35",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4556558235.2,
            "range": "± 55223608.97",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4756471690.8,
            "range": "± 130883459.01",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4364954822.1,
            "range": "± 51582163.22",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4454615294.1,
            "range": "± 59018128.08",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4578838057.2,
            "range": "± 71283597.36",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4319586202.3,
            "range": "± 63956855.36",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4344395775.2,
            "range": "± 31597537.08",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4335579035.9,
            "range": "± 37837333.25",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4313041443.3,
            "range": "± 111667488.7",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7992511669,
            "range": "± 115333510.88",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4535803317.1,
            "range": "± 41152979.73",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5297693610.2,
            "range": "± 65927238.41",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6129143655.1,
            "range": "± 80859319.22",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 22936.53,
            "range": "± 419.24",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 410524.95,
            "range": "± 37514.86",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 14723.16,
            "range": "± 4847.79",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 46580.35,
            "range": "± 15160.48",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 67481.24,
            "range": "± 2197.29",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 113053.88,
            "range": "± 20492.77",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 7193260.55,
            "range": "± 1003594.73",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_10",
            "value": 12531255.43,
            "range": "± 891745.18",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_50",
            "value": 36898534.71,
            "range": "± 14095929.65",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 199.28,
            "range": "± 8.03",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 1519.71,
            "range": "± 107.39",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 788.7,
            "range": "± 67.9",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 5481.05,
            "range": "± 256.04",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 54400.07,
            "range": "± 2117",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 27206.43,
            "range": "± 705.36",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 17292.64,
            "range": "± 2306.49",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 34389.17,
            "range": "± 12853.47",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 48289.28,
            "range": "± 16152.63",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1229.2,
            "range": "± 223.94",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 10.11,
            "range": "± 0.21",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 845.64,
            "range": "± 28.44",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 867.46,
            "range": "± 7.32",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 776.62,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 503.4,
            "range": "± 36.05",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 384.52,
            "range": "± 9.46",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 7731888.27,
            "range": "± 3520814.01",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3550647.21,
            "range": "± 487503.41",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 10443.73,
            "range": "± 687.88",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 821.54,
            "range": "± 55.61",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100000_k10000",
            "value": 2314367.05,
            "range": "± 180153.57",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n10000_k1000",
            "value": 225064.6,
            "range": "± 500111.7",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 16513.32,
            "range": "± 411.16",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1075.18,
            "range": "± 60.1",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 22.87,
            "range": "± 0.3",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 17551.8,
            "range": "± 396.99",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 7670.94,
            "range": "± 183.65",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 36765.3,
            "range": "± 824.62",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 50.27,
            "range": "± 0.6",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 4816.11,
            "range": "± 163.58",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 3252.31,
            "range": "± 65.38",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 9004.22,
            "range": "± 661.45",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 1782.74,
            "range": "± 83.39",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 15190.85,
            "range": "± 356.37",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 143874.9,
            "range": "± 2592.79",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 1872173.84,
            "range": "± 17426.3",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 112663.37,
            "range": "± 1504.03",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 8891.48,
            "range": "± 71.37",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 650.31,
            "range": "± 29.32",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 208792.8,
            "range": "± 12230.06",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 19176.75,
            "range": "± 1048.31",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 1733.43,
            "range": "± 150.49",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 256.8,
            "range": "± 3.68",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 113612.48,
            "range": "± 6247.47",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 112697.54,
            "range": "± 2803.83",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 112188.94,
            "range": "± 8707.88",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 109930.69,
            "range": "± 4138.91",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 9647.11,
            "range": "± 243.18",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 10453.56,
            "range": "± 254.95",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 19641.35,
            "range": "± 554.86",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 61358.43,
            "range": "± 1432.22",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 32405.94,
            "range": "± 515.78",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Dominik Oswald",
            "username": "d-oit",
            "email": "6849456+d-oit@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "37f9cafa060cf51fe9e744682df5acbf19ec55c4",
          "message": "test(config): give the default profile one retry and a 240s base cap (#1035)\n\n* test(config): give the default profile one retry and a 240s base cap\n\nPer-test overrides fixed the three CLI-wave tests but not the class: on the\nnext full run a different test\n(`do-memory-cli config::types::simple_config_tests::test_simple_config_with_cloud_platform`,\n37s in earlier runs) was starved past the 120s base cap while the CLI waves\nheld all four cores for ~6 minutes.\n\n`profile.default` — the only profile that gates local releases and the only\none with no retries and the tightest cap — now uses `retries = 1` and\n`slow-timeout = { period = \"120s\", terminate-after = 2 }` (240s). This matches\nthe precedent in `ci` (2 retries) and `nightly` (1) and cannot mask a\ndeterministic failure: retries only absorb transient starvation, and the retry\nruns serially after the parallel pass.\n\nVerified: `cargo nextest run --all` 3956/3956 passed, exit 0, 589s, no test\nneeded its retry to be reported as flaky.\n\n* docs(lessons): record why the default profile needed retries, not just caps\n\nLESSON-028 now carries the second iteration: per-test overrides fixed three\nCLI-wave tests but not the class, and the next full run starved a 37s unit\ntest past the base cap. Records the reasoning for `retries = 1` (cannot mask a\ndeterministic failure; the retry runs serially after the parallel pass) and\nthe final green result.\n\n---------\n\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-20T10:10:50Z",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/37f9cafa060cf51fe9e744682df5acbf19ec55c4"
        },
        "date": 1789960892401,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48182.35,
            "range": "± 114.65",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18209.45,
            "range": "± 112.67",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 179198.71,
            "range": "± 875.94",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31285.06,
            "range": "± 146.79",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 156194.72,
            "range": "± 1226.34",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 40066944.73,
            "range": "± 657654.57",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 381388558.2,
            "range": "± 5035433.85",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 188127071.63,
            "range": "± 2198381.51",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 383572610.8,
            "range": "± 7628892.35",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 75018916.19,
            "range": "± 1756739.61",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 22827679.75,
            "range": "± 510074.93",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 183.38,
            "range": "± 1.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 270.54,
            "range": "± 920.37",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9.97,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 20.41,
            "range": "± 0.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2177624.06,
            "range": "± 35604.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 402.49,
            "range": "± 28.86",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3661.23,
            "range": "± 41.41",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 194.53,
            "range": "± 2.74",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 233.6,
            "range": "± 3.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 284.41,
            "range": "± 3.42",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 211.06,
            "range": "± 1.31",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3711.72,
            "range": "± 26.97",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40943.53,
            "range": "± 168.39",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 429412.04,
            "range": "± 6158.81",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2214580.27,
            "range": "± 7418.62",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 184.68,
            "range": "± 1.78",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12850.28,
            "range": "± 56.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1239.69,
            "range": "± 5.45",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 390.69,
            "range": "± 9.47",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 427.35,
            "range": "± 12.05",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 476.82,
            "range": "± 17.13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 404.78,
            "range": "± 10.22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 550.03,
            "range": "± 9.51",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 733.9,
            "range": "± 14.47",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.2,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58.14,
            "range": "± 0.2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2754.77,
            "range": "± 34.13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74111.68,
            "range": "± 426.51",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 330.33,
            "range": "± 1.57",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7524.77,
            "range": "± 14.3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1423.62,
            "range": "± 11.25",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37019.93,
            "range": "± 324.15",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 161462.96,
            "range": "± 1278.03",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41708.07,
            "range": "± 212.9",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 661011.41,
            "range": "± 7306.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10740.19,
            "range": "± 101.44",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 9259475.26,
            "range": "± 94849.9",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9259477.83,
            "range": "± 144356.73",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6.68,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6.69,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.68,
            "range": "± 0.49",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 148955.5,
            "range": "± 1569.38",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 81151.55,
            "range": "± 975.02",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4360094670.7,
            "range": "± 50894504.86",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4515966636.8,
            "range": "± 40162523.46",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4652371935.2,
            "range": "± 37845745.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4955347146.8,
            "range": "± 61383566.41",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4362664061.9,
            "range": "± 69561520",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4512978088,
            "range": "± 58106079.26",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4670096611.7,
            "range": "± 33219847.64",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4742202886,
            "range": "± 59716925.26",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4343339682.1,
            "range": "± 36146322.49",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4425809718.3,
            "range": "± 29407648.81",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4501401889.5,
            "range": "± 44137304.07",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7954656623.9,
            "range": "± 80853233.33",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4569105260.4,
            "range": "± 41053191.62",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5288722758.5,
            "range": "± 54171030.01",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6153324655.2,
            "range": "± 63111927.32",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34260.47,
            "range": "± 620.16",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 835033.16,
            "range": "± 2142.93",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 16318.67,
            "range": "± 1142.02",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 48433.87,
            "range": "± 2226.85",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 97528.83,
            "range": "± 2518.19",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 147520.15,
            "range": "± 1609.41",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 334.52,
            "range": "± 2.11",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2770.11,
            "range": "± 28.53",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1420.83,
            "range": "± 7.09",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7512.09,
            "range": "± 35.44",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74609.31,
            "range": "± 1021.22",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 36992.61,
            "range": "± 143.95",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23546.77,
            "range": "± 57.09",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 40615.17,
            "range": "± 2989.25",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 47574.74,
            "range": "± 1268.36",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2206.85,
            "range": "± 7.29",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.06,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 993.27,
            "range": "± 3.36",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 977.77,
            "range": "± 6.79",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 948.53,
            "range": "± 7.61",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 663.54,
            "range": "± 11.2",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 525.99,
            "range": "± 10.79",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5691432.6,
            "range": "± 164617",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2886938.9,
            "range": "± 204753.93",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 18688.91,
            "range": "± 89.13",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 916.52,
            "range": "± 2.08",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30.89,
            "range": "± 0.56",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28912.9,
            "range": "± 234.09",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13256.92,
            "range": "± 51.97",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 59353.77,
            "range": "± 446.7",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30.14,
            "range": "± 0.07",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7592.34,
            "range": "± 101.34",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5093.33,
            "range": "± 72.99",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13785.2,
            "range": "± 260.7",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3893.92,
            "range": "± 15.56",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 35553.05,
            "range": "± 146.25",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 349645.59,
            "range": "± 1567.39",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 189000.93,
            "range": "± 1080.84",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14581.64,
            "range": "± 86.15",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1013.12,
            "range": "± 11.31",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 31436.51,
            "range": "± 148.01",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3096.11,
            "range": "± 13.86",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 420.82,
            "range": "± 36.93",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 53017.61,
            "range": "± 572.25",
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
          "id": "dd729e57f627c7133f128373ad34dc8d11a4ee76",
          "message": "chore(release): bump workspace to 0.1.42 after shipping v0.1.41 (#1039)\n\n* chore(release): bump workspace to 0.1.42 after shipping v0.1.41\n\nv0.1.41 is tagged and published (tag `v0.1.41` on `37f9cafa`, GitHub Release\nwith dist artifacts for five targets, drift issue #1020 closed). Leaving the\nworkspace at 0.1.41 while the tag matches would fire the critical\n`version_not_advanced` drift reason on the next main push, so the workspace\nmoves to the next patch.\n\n- `Cargo.toml` workspace version 0.1.41 -> 0.1.42 (all members inherit via\n  `version.workspace = true`); `Cargo.lock` updated with\n  `cargo update --workspace --offline` (only the nine member version strings\n  change)\n- `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/STATUS/CURRENT.md`,\n  `plans/README.md`: Released Version = v0.1.41 (latest tag), Workspace =\n  0.1.42 (post-v0.1.41 bump), refreshed release-state prose\n\nVerified: `./scripts/check-release-drift.sh` reports `severity=clean`\n(`reason=within_cadence`) and `./scripts/check-docs-integrity.sh` passes.\n`verify-release-state.sh --check-unreleased` intentionally still expects the\ndocs to match the *next* release and is re-satisfied by the next release-prep.\n\n* docs(changelog): record the #1033 bound under Unreleased\n\n`DomainIndex::get_recent_episodes` was merged to main after the v0.1.41\nrelease-prep PR (#1023) froze the 0.1.41 notes, so it is not in that\nrelease's notes. Record it under `## [Unreleased]` with the observable\nbehaviour (limit capped at `MAX_QUERY_LIMIT`) so the next release-prep picks\nit up.",
          "timestamp": "2026-09-21T17:15:19+02:00",
          "tree_id": "39145a1e6d2a5ce36c766d1954ef0970687f5614",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/dd729e57f627c7133f128373ad34dc8d11a4ee76"
        },
        "date": 1790006882997,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48298.57,
            "range": "± 144.93",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18470.9,
            "range": "± 344.97",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 179784.16,
            "range": "± 3137.1",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 31566.11,
            "range": "± 118.76",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 158137.08,
            "range": "± 9678.02",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 43198046.17,
            "range": "± 1134064.66",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 389037293.45,
            "range": "± 8287698.3",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 196215066.93,
            "range": "± 6328254.66",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 391488290.5,
            "range": "± 6760161.25",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 80405840.77,
            "range": "± 1496366.59",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 25167480.98,
            "range": "± 1005243.18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 182.12,
            "range": "± 3.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 319.56,
            "range": "± 1067.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9.99,
            "range": "± 0.08",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 18.6,
            "range": "± 0.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2310779.27,
            "range": "± 44117.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 540.53,
            "range": "± 31.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3714.64,
            "range": "± 49.56",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 195.11,
            "range": "± 2.13",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 231.14,
            "range": "± 1.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 280.88,
            "range": "± 2.36",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 206.9,
            "range": "± 0.92",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3720.46,
            "range": "± 27.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 40776.56,
            "range": "± 211.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 427842.9,
            "range": "± 2994.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2209454.19,
            "range": "± 18880.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 186.84,
            "range": "± 5.77",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12784.51,
            "range": "± 53.17",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1233.42,
            "range": "± 16.65",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 390.99,
            "range": "± 12.53",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 442.98,
            "range": "± 24.09",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 486.65,
            "range": "± 26.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 417.57,
            "range": "± 19.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 548.44,
            "range": "± 6.52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 737.53,
            "range": "± 15.7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.24,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58.46,
            "range": "± 1.4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2789.4,
            "range": "± 22.46",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74152.33,
            "range": "± 312.86",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 315.05,
            "range": "± 0.98",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7472.06,
            "range": "± 9.59",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1418.31,
            "range": "± 4.88",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 36973.76,
            "range": "± 135.34",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 162656.99,
            "range": "± 1345.12",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 41879.64,
            "range": "± 128.4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 662570.62,
            "range": "± 3441.29",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10731.32,
            "range": "± 281.94",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 10125732.98,
            "range": "± 317637.77",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 9802143.55,
            "range": "± 387359.84",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8.29,
            "range": "± 0.9",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8.29,
            "range": "± 0.9",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8.28,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8.31,
            "range": "± 0.91",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 152001.06,
            "range": "± 1738.82",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 82364.01,
            "range": "± 931.35",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4401625854.9,
            "range": "± 33420316.14",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4522637170,
            "range": "± 29328678.25",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4712952510.1,
            "range": "± 65929264.88",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5002892949.9,
            "range": "± 52684460.57",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4440456644.2,
            "range": "± 55416906.92",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4552510543.2,
            "range": "± 46275695.36",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4723970945.9,
            "range": "± 40696748.9",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4782980488.5,
            "range": "± 65434286.53",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4390693112.3,
            "range": "± 48675190.35",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4492603310.4,
            "range": "± 48894982.99",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4578851894.8,
            "range": "± 37582887.47",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8128558728.2,
            "range": "± 91874662.32",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4605601879.1,
            "range": "± 52293813.8",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5318575604.9,
            "range": "± 87150076.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6172783519.2,
            "range": "± 60207100.06",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 35284.33,
            "range": "± 296.66",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 902428.35,
            "range": "± 12276.93",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 28006.41,
            "range": "± 9194.9",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 70986.5,
            "range": "± 23048.39",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 172795.96,
            "range": "± 62873.46",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 194914.57,
            "range": "± 46085.85",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 323.09,
            "range": "± 2.61",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2771.86,
            "range": "± 7.74",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1450.3,
            "range": "± 7.79",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7467.51,
            "range": "± 12.88",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74716.87,
            "range": "± 2274.46",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37071.12,
            "range": "± 550.97",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 23953.37,
            "range": "± 122.59",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 60858.27,
            "range": "± 21088.3",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 52137.63,
            "range": "± 6529.82",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2224.28,
            "range": "± 5.39",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.19,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1055.44,
            "range": "± 4.23",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1062.38,
            "range": "± 10.72",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1008.14,
            "range": "± 12.56",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 698.45,
            "range": "± 22.34",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 554.88,
            "range": "± 11.93",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6109371.82,
            "range": "± 341513.04",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 3095465.11,
            "range": "± 251612.52",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 19099.71,
            "range": "± 898.76",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 977.94,
            "range": "± 20.16",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 30.6,
            "range": "± 0.22",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 29003.42,
            "range": "± 114.7",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13065.91,
            "range": "± 41.81",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 59906.86,
            "range": "± 639.75",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30.06,
            "range": "± 0.07",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7651.99,
            "range": "± 83.94",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5122.54,
            "range": "± 55.95",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13857.33,
            "range": "± 145.99",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3935.94,
            "range": "± 114.46",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 36265.8,
            "range": "± 893.4",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 354515.58,
            "range": "± 7821.85",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 55388.55,
            "range": "± 540.55",
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
          "id": "0fa9499325597379e08a849493d081b61b0bc416",
          "message": "ci(deps): bump the actions-all group with 5 updates (#1037)\n\nBumps the actions-all group with 5 updates:\n\n| Package | From | To |\n| --- | --- | --- |\n| [taiki-e/install-action](https://github.com/taiki-e/install-action) | `2.87.5` | `2.87.15` |\n| [benchmark-action/github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) | `1.22.1` | `1.22.2` |\n| [jlumbroso/free-disk-space](https://github.com/jlumbroso/free-disk-space) | `1.3.1` | `2.0.0` |\n| [codecov/codecov-action](https://github.com/codecov/codecov-action) | `7.0.0` | `7.1.1` |\n| [reviewdog/action-actionlint](https://github.com/reviewdog/action-actionlint) | `1.73.4` | `1.76.0` |\n\n\nUpdates `taiki-e/install-action` from 2.87.5 to 2.87.15\n- [Release notes](https://github.com/taiki-e/install-action/releases)\n- [Changelog](https://github.com/taiki-e/install-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/taiki-e/install-action/compare/5bf6ce016fd2e72eefc647cbca1e4213f65955b8...4076c08d76dba979c11a7285295b0716c1d67908)\n\nUpdates `benchmark-action/github-action-benchmark` from 1.22.1 to 1.22.2\n- [Release notes](https://github.com/benchmark-action/github-action-benchmark/releases)\n- [Changelog](https://github.com/benchmark-action/github-action-benchmark/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/benchmark-action/github-action-benchmark/compare/52576c92bccf6ac60c8223ec7eb2565637cae9ba...4322e5726e6334590d251fc4f92bec0efafc45dc)\n\nUpdates `jlumbroso/free-disk-space` from 1.3.1 to 2.0.0\n- [Release notes](https://github.com/jlumbroso/free-disk-space/releases)\n- [Commits](https://github.com/jlumbroso/free-disk-space/compare/54081f138730dfa15788a46383842cd2f914a1be...ceedf095f4ec1a097402bc6bd80831f2e1a6fde6)\n\nUpdates `codecov/codecov-action` from 7.0.0 to 7.1.1\n- [Release notes](https://github.com/codecov/codecov-action/releases)\n- [Changelog](https://github.com/codecov/codecov-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/codecov/codecov-action/compare/fb8b3582c8e4def4969c97caa2f19720cb33a72f...303a32d7a59b442fa8d48b6a1cc6825c09c847a5)\n\nUpdates `reviewdog/action-actionlint` from 1.73.4 to 1.76.0\n- [Release notes](https://github.com/reviewdog/action-actionlint/releases)\n- [Commits](https://github.com/reviewdog/action-actionlint/compare/d290e336d5a743810aef4404f757dc862276d2ae...320fcdd9c860767cf17fab3b20e22e739d5d02b8)\n\n---\nupdated-dependencies:\n- dependency-name: taiki-e/install-action\n  dependency-version: 2.87.15\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: actions-all\n- dependency-name: benchmark-action/github-action-benchmark\n  dependency-version: 1.22.2\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: actions-all\n- dependency-name: jlumbroso/free-disk-space\n  dependency-version: 2.0.0\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n  dependency-group: actions-all\n- dependency-name: codecov/codecov-action\n  dependency-version: 7.1.1\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: actions-all\n- dependency-name: reviewdog/action-actionlint\n  dependency-version: 1.76.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: actions-all\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-21T19:08:52+02:00",
          "tree_id": "ca27f7fc79ad53327fa224c6066664b9d854a040",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/0fa9499325597379e08a849493d081b61b0bc416"
        },
        "date": 1790013941553,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 39656.63,
            "range": "± 107.97",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14364.2,
            "range": "± 45.72",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 154981.43,
            "range": "± 5573.24",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25463.28,
            "range": "± 64.76",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 122117.27,
            "range": "± 482.93",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 109239197.02,
            "range": "± 45408879.08",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 459373193.4,
            "range": "± 140083706.37",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 419227286.4,
            "range": "± 218112981.89",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 1006258437.3,
            "range": "± 690944213.86",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 200046196.93,
            "range": "± 121335773.33",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 92821009.12,
            "range": "± 36965480.5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 153.85,
            "range": "± 3.65",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 326.52,
            "range": "± 1804.57",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 8.75,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 16.63,
            "range": "± 0.32",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1662363.36,
            "range": "± 83417.7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 386.2,
            "range": "± 46.38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2976.67,
            "range": "± 56.62",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 154.53,
            "range": "± 2.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 195.29,
            "range": "± 17.93",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 233.83,
            "range": "± 13.2",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 174.34,
            "range": "± 2.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2993.52,
            "range": "± 15.18",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 32998.88,
            "range": "± 590.28",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 347103.09,
            "range": "± 8524.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1743573.63,
            "range": "± 4339.8",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 149.52,
            "range": "± 3.29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11171.21,
            "range": "± 63.7",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1055.12,
            "range": "± 2.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 309.28,
            "range": "± 20.31",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 341.39,
            "range": "± 23.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 379.62,
            "range": "± 17.64",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 315.65,
            "range": "± 13.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 462.99,
            "range": "± 10.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 616.04,
            "range": "± 11.99",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 5.4,
            "range": "± 0.06",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 48.59,
            "range": "± 0.26",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 48.56,
            "range": "± 0.05",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 48.62,
            "range": "± 0.43",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_5000",
            "value": 48.7,
            "range": "± 0.88",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2192.1,
            "range": "± 6.2",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 64180.41,
            "range": "± 898.7",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 238.54,
            "range": "± 2.82",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6461.67,
            "range": "± 28.21",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1115.36,
            "range": "± 2.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 32067.24,
            "range": "± 776.42",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 126527.32,
            "range": "± 677.54",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 32936.05,
            "range": "± 418.11",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 510694.04,
            "range": "± 2614.35",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8264.55,
            "range": "± 290.61",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 19407091.63,
            "range": "± 9432607.27",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 24236687.76,
            "range": "± 23779258.78",
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
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.04,
            "range": "± 0.09",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 103354.72,
            "range": "± 1463.07",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 57232.52,
            "range": "± 469.54",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 11412641192.5,
            "range": "± 2590022259.97",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 8894280017.3,
            "range": "± 2412427621.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 19955033587.1,
            "range": "± 5013282083.67",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 7504939107,
            "range": "± 1409662025.94",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 7060943740.1,
            "range": "± 1712491933.04",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 12741400280.8,
            "range": "± 3088928029.25",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 27798.72,
            "range": "± 119.13",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 680915.39,
            "range": "± 3439.35",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 14452.48,
            "range": "± 1824.56",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 41284.65,
            "range": "± 4384.54",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 82458.58,
            "range": "± 10307.55",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 133737.42,
            "range": "± 21211.36",
            "unit": "ns/iter"
          },
          {
            "name": "episode_creation_1",
            "value": 15031090.14,
            "range": "± 9542814.45",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 239.08,
            "range": "± 0.51",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2175.41,
            "range": "± 5.32",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1111.08,
            "range": "± 3.85",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6467.59,
            "range": "± 21.02",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 63564.63,
            "range": "± 81.22",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 31959.22,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 18847.46,
            "range": "± 188.36",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 34386.12,
            "range": "± 6560.66",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 42048.06,
            "range": "± 4352.31",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1567.74,
            "range": "± 5.56",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 5.36,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 816.24,
            "range": "± 8.05",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 809.16,
            "range": "± 6.45",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 774.07,
            "range": "± 3.81",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 514.85,
            "range": "± 9.22",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 412.84,
            "range": "± 6.96",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 11076021.19,
            "range": "± 10894543.21",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 9154521.88,
            "range": "± 10376314.57",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14973.67,
            "range": "± 113.99",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 761.51,
            "range": "± 0.4",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27.76,
            "range": "± 0.55",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 23240.67,
            "range": "± 224.58",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10563.46,
            "range": "± 148.12",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 47843.38,
            "range": "± 818.2",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 27.28,
            "range": "± 0.07",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 5947.54,
            "range": "± 35.46",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4035.48,
            "range": "± 41.81",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 11017.72,
            "range": "± 117.37",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3080.38,
            "range": "± 7.67",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 28574.41,
            "range": "± 826.97",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 278117.28,
            "range": "± 2868.64",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2643046.12,
            "range": "± 74622.99",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 164335.54,
            "range": "± 678.11",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 12867.28,
            "range": "± 32.07",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 911.54,
            "range": "± 7.6",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 305767.72,
            "range": "± 23198.61",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 28684.95,
            "range": "± 72.46",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 3078.09,
            "range": "± 19.48",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 382.73,
            "range": "± 5.89",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 164291.57,
            "range": "± 619.32",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 164956.63,
            "range": "± 2179.7",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 165004.81,
            "range": "± 1557.38",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 165076.39,
            "range": "± 1144.79",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 16318.3,
            "range": "± 57.26",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 17409.83,
            "range": "± 135.08",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 30263.44,
            "range": "± 201.39",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 93279.96,
            "range": "± 221.43",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 38919.87,
            "range": "± 995.51",
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
          "id": "4f7a6d189d6f9a8594971f952e9f0ef272ff17a0",
          "message": "chore(deps): bump the rust-patch-minor group with 4 updates (#1038)\n\nBumps the rust-patch-minor group with 4 updates: [redb](https://github.com/cberner/redb), [clap](https://github.com/clap-rs/clap), [jsonwebtoken](https://github.com/Keats/jsonwebtoken) and [clap_complete](https://github.com/clap-rs/clap).\n\n\nUpdates `redb` from 4.2.0 to 4.3.0\n- [Release notes](https://github.com/cberner/redb/releases)\n- [Changelog](https://github.com/cberner/redb/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/cberner/redb/compare/v4.2.0...v4.3.0)\n\nUpdates `clap` from 4.6.6 to 4.6.7\n- [Release notes](https://github.com/clap-rs/clap/releases)\n- [Changelog](https://github.com/clap-rs/clap/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.6.6...clap_complete-v4.6.7)\n\nUpdates `jsonwebtoken` from 11.0.0 to 11.1.0\n- [Changelog](https://github.com/Keats/jsonwebtoken/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/Keats/jsonwebtoken/compare/v11.0.0...v11.1.0)\n\nUpdates `clap_complete` from 4.6.9 to 4.6.11\n- [Release notes](https://github.com/clap-rs/clap/releases)\n- [Changelog](https://github.com/clap-rs/clap/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.6.9...clap_complete-v4.6.11)\n\n---\nupdated-dependencies:\n- dependency-name: redb\n  dependency-version: 4.3.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-patch-minor\n- dependency-name: clap\n  dependency-version: 4.6.7\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n- dependency-name: jsonwebtoken\n  dependency-version: 11.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: rust-patch-minor\n- dependency-name: clap_complete\n  dependency-version: 4.6.11\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: rust-patch-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: d.o. <242170972+d-o-hub@users.noreply.github.com>",
          "timestamp": "2026-09-21T20:20:07+02:00",
          "tree_id": "d8aa89978fe2fbfa96ebae18e36290f3f22b4c95",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/4f7a6d189d6f9a8594971f952e9f0ef272ff17a0"
        },
        "date": 1790017875983,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 62989.71,
            "range": "± 430.38",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 22861.93,
            "range": "± 249.39",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 241231.43,
            "range": "± 1754.33",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 40795.75,
            "range": "± 375.15",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 198649.52,
            "range": "± 1138.98",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 68589428.81,
            "range": "± 7014461.3",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 543692402.3,
            "range": "± 32212855.6",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 291986752.15,
            "range": "± 25883490.86",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 519377513.9,
            "range": "± 21117436.25",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 125107716.15,
            "range": "± 15883333.53",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 37597257.44,
            "range": "± 4370584.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 174.78,
            "range": "± 1.09",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 324.12,
            "range": "± 1352.72",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 31.09,
            "range": "± 0.23",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 36.95,
            "range": "± 0.25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2484076.41,
            "range": "± 29468.52",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 517.35,
            "range": "± 54.68",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3699.13,
            "range": "± 40.32",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 194.76,
            "range": "± 1.26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 302.87,
            "range": "± 3.59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 427.35,
            "range": "± 3.26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 244.94,
            "range": "± 10.9",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 4395.01,
            "range": "± 32.24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 48309.95,
            "range": "± 302.04",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 503283.27,
            "range": "± 3441.16",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2481059.79,
            "range": "± 14354.6",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 195.34,
            "range": "± 1.65",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11727.7,
            "range": "± 128.96",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1130.25,
            "range": "± 8.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 392.26,
            "range": "± 16.62",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 494.93,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 611.48,
            "range": "± 39.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 435.3,
            "range": "± 15.48",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 543.65,
            "range": "± 9.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 727.19,
            "range": "± 30.55",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 20.35,
            "range": "± 0.13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 47.34,
            "range": "± 0.35",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2690.34,
            "range": "± 18.01",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 75979.37,
            "range": "± 415.09",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 279.59,
            "range": "± 5.37",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7647.66,
            "range": "± 39.36",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1395.39,
            "range": "± 8.26",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 38028.62,
            "range": "± 185.21",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 206929.7,
            "range": "± 5388.67",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 51766.01,
            "range": "± 662.09",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 831842.56,
            "range": "± 11069.3",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 12670.89,
            "range": "± 162.69",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 14671583.16,
            "range": "± 1839115.04",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 15713018.45,
            "range": "± 1634441",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.25,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 6.26,
            "range": "± 0.01",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 6.23,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 6.26,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 6.24,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 107151.73,
            "range": "± 2601.89",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 72176.84,
            "range": "± 935.77",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 5978437011.8,
            "range": "± 298766640.52",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 5727168992.4,
            "range": "± 371015400.28",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 6121998798.2,
            "range": "± 311627528.67",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5848402389,
            "range": "± 244486946.35",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 5372700222.3,
            "range": "± 271429833.23",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 6045913801.6,
            "range": "± 247059386.87",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5385607407.3,
            "range": "± 427409762.22",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 10481287177.7,
            "range": "± 636249924",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 5843844582.9,
            "range": "± 443974385.65",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 6606535157.4,
            "range": "± 458697426.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 7469821150.8,
            "range": "± 292790773.96",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 23236.4,
            "range": "± 609.62",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 652250.75,
            "range": "± 14367.39",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 22450.35,
            "range": "± 26174.58",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 58970.03,
            "range": "± 12764.16",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 118864.06,
            "range": "± 11079.71",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 173164.74,
            "range": "± 9324.89",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 296.85,
            "range": "± 1.77",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2725.56,
            "range": "± 16.13",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1422.38,
            "range": "± 9.91",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7741.46,
            "range": "± 34.92",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 76600.09,
            "range": "± 484.03",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 38405.02,
            "range": "± 355.45",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 22828.31,
            "range": "± 340.51",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 54169.51,
            "range": "± 6165.77",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 58211.98,
            "range": "± 4281.95",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 3198.54,
            "range": "± 24.61",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 20.33,
            "range": "± 0.13",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 1276.33,
            "range": "± 4.88",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 1279.44,
            "range": "± 5.53",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 1151.07,
            "range": "± 9.13",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 704.21,
            "range": "± 21.83",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 545.66,
            "range": "± 10.81",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 9550814.61,
            "range": "± 1730887.45",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 4081622.28,
            "range": "± 563408.96",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 13964.5,
            "range": "± 216.33",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 1167.36,
            "range": "± 2.16",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 31.2,
            "range": "± 0.25",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 26860.85,
            "range": "± 381.5",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 12119.87,
            "range": "± 214.81",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 54929.92,
            "range": "± 905.44",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 73.13,
            "range": "± 1.26",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7676.8,
            "range": "± 75.48",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5059.32,
            "range": "± 53.07",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 14061.63,
            "range": "± 82.29",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3276.29,
            "range": "± 32.52",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 28035.12,
            "range": "± 196.57",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 278017.54,
            "range": "± 4582.45",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 59914.63,
            "range": "± 784.87",
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
          "id": "601b46ea75a8d19d64ef3a1d313347e4550acc42",
          "message": "Merge pull request #1041 from d-o-hub/feat/typed-retrieval-judge-1030\n\nfeat(retrieval): add typed semantic judge interface",
          "timestamp": "2026-09-23T14:53:31+02:00",
          "tree_id": "e0c318de38e5de842bd8cc492407c61c6e6a1a88",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/601b46ea75a8d19d64ef3a1d313347e4550acc42"
        },
        "date": 1790171138015,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 48860.13,
            "range": "± 137.68",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 18727.75,
            "range": "± 139.49",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 180084.64,
            "range": "± 583.22",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 32146.7,
            "range": "± 190.21",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 160416.49,
            "range": "± 6121.74",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 40725478.48,
            "range": "± 1845671.81",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 371237329.85,
            "range": "± 4894357.41",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 186036723.13,
            "range": "± 2807670.19",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 375660343.3,
            "range": "± 5759290.99",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 74125366.14,
            "range": "± 1564651.48",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 22496562.69,
            "range": "± 564270.06",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 180.92,
            "range": "± 1.98",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 273.93,
            "range": "± 942.92",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 9.99,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 19,
            "range": "± 0.47",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 2280638.94,
            "range": "± 41549.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 475.05,
            "range": "± 41.54",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 3660.86,
            "range": "± 35.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 199.49,
            "range": "± 3.01",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 230.85,
            "range": "± 1.26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 281.84,
            "range": "± 1.98",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 207.42,
            "range": "± 1.21",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 3842.58,
            "range": "± 25.74",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 42649.2,
            "range": "± 2312.43",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 445751.69,
            "range": "± 6521.99",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 2264215.39,
            "range": "± 25350.59",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 181.14,
            "range": "± 2.24",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 12858.66,
            "range": "± 49.42",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1247.91,
            "range": "± 3.93",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 384.38,
            "range": "± 10.3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 423.7,
            "range": "± 14.26",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 474.64,
            "range": "± 19.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 403.74,
            "range": "± 16.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 547.42,
            "range": "± 6.14",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 733.06,
            "range": "± 11.25",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 6.23,
            "range": "± 0.09",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 58.17,
            "range": "± 0.08",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2785.42,
            "range": "± 13.49",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 74644.14,
            "range": "± 513.82",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 334.81,
            "range": "± 12.59",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 7544.53,
            "range": "± 30.84",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1427.8,
            "range": "± 4.68",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 37210.84,
            "range": "± 69.13",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 165465.74,
            "range": "± 3733.84",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 43262.19,
            "range": "± 404.33",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 669178.43,
            "range": "± 5845.96",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 10863.52,
            "range": "± 105.23",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 8906306.54,
            "range": "± 261539.02",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 8871071.48,
            "range": "± 271455.16",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 8.28,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_10",
            "value": 8.28,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_100",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_5",
            "value": 8.27,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_50",
            "value": 8.28,
            "range": "± 0.89",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_1",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 150330.74,
            "range": "± 2181.9",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 81253.15,
            "range": "± 552.63",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_16_read_latest@16",
            "value": 4742250322.7,
            "range": "± 60578193.05",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 4160482458.5,
            "range": "± 52238866.75",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4301587156.9,
            "range": "± 32906783.62",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_8_read_latest@8",
            "value": 4414466109.3,
            "range": "± 42011554.38",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4741904311.7,
            "range": "± 54294488.08",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 4163365294,
            "range": "± 32937328.47",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4273304697.3,
            "range": "± 42919429",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 4429875197.7,
            "range": "± 49380043.06",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 4471239857.5,
            "range": "± 35471071.1",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4124699576.6,
            "range": "± 38263030.76",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4192659524,
            "range": "± 50772168.07",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 4285976094.6,
            "range": "± 46247618.53",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 7748814781.6,
            "range": "± 86942906.49",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4325332397.4,
            "range": "± 54861472.34",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5019029242.5,
            "range": "± 69942761.75",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 5877191246,
            "range": "± 49584695.98",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 34913.38,
            "range": "± 755.2",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 775999.34,
            "range": "± 6078.85",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 16254.82,
            "range": "± 1136.33",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 47477.66,
            "range": "± 1867.75",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 95212.96,
            "range": "± 1800.55",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 149241.7,
            "range": "± 5993.31",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 339.05,
            "range": "± 1.15",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2785.31,
            "range": "± 18.95",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1437.38,
            "range": "± 26.5",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 7543.09,
            "range": "± 21.04",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 74339.92,
            "range": "± 190.43",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 37174.3,
            "range": "± 107.19",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 24028.92,
            "range": "± 345.99",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 39405.51,
            "range": "± 1063.96",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 47117.37,
            "range": "± 949.11",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 2198.41,
            "range": "± 13.85",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 6.15,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 965.95,
            "range": "± 3.56",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 960.88,
            "range": "± 4.83",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 925.17,
            "range": "± 1.32",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 654.8,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 522.92,
            "range": "± 7.52",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 5391460.39,
            "range": "± 337273.45",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 2800072.45,
            "range": "± 161189.24",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 19473.34,
            "range": "± 393.32",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 905.6,
            "range": "± 1.05",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 31.19,
            "range": "± 0.72",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 28741.67,
            "range": "± 188.24",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 13035.14,
            "range": "± 56.97",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 58874.68,
            "range": "± 213.69",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 30.03,
            "range": "± 0.06",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 7524.68,
            "range": "± 68.22",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 5065.96,
            "range": "± 59.01",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 13715.67,
            "range": "± 89.96",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 3664.14,
            "range": "± 14.15",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 33306.73,
            "range": "± 169.65",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 326976.42,
            "range": "± 1787.06",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 188094.67,
            "range": "± 751.85",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 14318.53,
            "range": "± 64.04",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1093.16,
            "range": "± 22.06",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2851.72,
            "range": "± 16.09",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 267.85,
            "range": "± 2.41",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 53474.1,
            "range": "± 1144.13",
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
          "id": "1461d61dc5da70603a8e2ea0286e7507827d8e00",
          "message": "Merge pull request #1042 from d-o-hub/feat/semantic-rerank-1031\n\nfeat(retrieval): semantic shortlist rerank with offline eval comparison",
          "timestamp": "2026-09-24T19:04:43+02:00",
          "tree_id": "c838e9b1af10e7c07dd50642ab9237afd761f5e3",
          "url": "https://github.com/d-o-hub/rust-self-learning-memory/commit/1461d61dc5da70603a8e2ea0286e7507827d8e00"
        },
        "date": 1790272754960,
        "tool": "cargo",
        "benches": [
          {
            "name": "broadcast_fan_out_fan_out_10_receivers",
            "value": 39151.26,
            "range": "± 90.23",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_1_receivers",
            "value": 14201.54,
            "range": "± 46.14",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_50_receivers",
            "value": 152067.67,
            "range": "± 677.27",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_fan_out_fan_out_5_receivers",
            "value": 25172.3,
            "range": "± 401.83",
            "unit": "ns/iter"
          },
          {
            "name": "broadcast_single_receiver_send_1000_events",
            "value": 122535.58,
            "range": "± 1503.98",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_10",
            "value": 80235853.93,
            "range": "± 64829526.75",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_100",
            "value": 893440157.7,
            "range": "± 604996608.68",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_episode_operations_50",
            "value": 262245035.8,
            "range": "± 125380435.37",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_100",
            "value": 557773786.8,
            "range": "± 347024064.64",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_20",
            "value": 98986201.1,
            "range": "± 43755454.36",
            "unit": "ns/iter"
          },
          {
            "name": "bulk_pattern_extraction_5",
            "value": 62980242.95,
            "range": "± 44746085.76",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_hit",
            "value": 152.59,
            "range": "± 2.38",
            "unit": "ns/iter"
          },
          {
            "name": "cache_basic_cache_miss",
            "value": 287.56,
            "range": "± 1416.93",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_all",
            "value": 8.74,
            "range": "± 0.02",
            "unit": "ns/iter"
          },
          {
            "name": "cache_cleanup_clear_connection",
            "value": 16.65,
            "range": "± 0.32",
            "unit": "ns/iter"
          },
          {
            "name": "cache_concurrent_concurrent_access",
            "value": 1643893.93,
            "range": "± 79639.91",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction",
            "value": 416.87,
            "range": "± 50.28",
            "unit": "ns/iter"
          },
          {
            "name": "cache_eviction_lru_eviction",
            "value": 2999.64,
            "range": "± 23.29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_1",
            "value": 154.17,
            "range": "± 1.73",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_10",
            "value": 190.04,
            "range": "± 1.29",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_20",
            "value": 229.96,
            "range": "± 1.5",
            "unit": "ns/iter"
          },
          {
            "name": "cache_hit_5",
            "value": 174.46,
            "range": "± 5.22",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_10",
            "value": 2930.36,
            "range": "± 21.94",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_100",
            "value": 32679.89,
            "range": "± 343.3",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_1000",
            "value": 342250.36,
            "range": "± 1872.58",
            "unit": "ns/iter"
          },
          {
            "name": "cache_invalidation_5000",
            "value": 1734382.19,
            "range": "± 15223.44",
            "unit": "ns/iter"
          },
          {
            "name": "cache_miss",
            "value": 149.59,
            "range": "± 3.53",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_100_connections",
            "value": 11141.63,
            "range": "± 61.46",
            "unit": "ns/iter"
          },
          {
            "name": "cache_multi_conn_10_connections",
            "value": 1058.29,
            "range": "± 12.1",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_1",
            "value": 307.66,
            "range": "± 12.84",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_10",
            "value": 335.22,
            "range": "± 16.83",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_20",
            "value": 379.38,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cache_put_5",
            "value": 314.2,
            "range": "± 10.96",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_parameterized_queries",
            "value": 459.85,
            "range": "± 6.8",
            "unit": "ns/iter"
          },
          {
            "name": "cache_sql_patterns_repeated_queries",
            "value": 618.18,
            "range": "± 11.15",
            "unit": "ns/iter"
          },
          {
            "name": "cache_statistics_stats_calculation",
            "value": 5.39,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_100",
            "value": 48.54,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_1000",
            "value": 48.54,
            "range": "± 0.04",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_check_efficiency_500",
            "value": 48.55,
            "range": "± 0.06",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_LRU",
            "value": 2201.86,
            "range": "± 4.88",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_1000episodes_RelevanceWeighted",
            "value": 64081.53,
            "range": "± 627.74",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_LRU",
            "value": 238.69,
            "range": "± 5.06",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_100episodes_RelevanceWeighted",
            "value": 6445.25,
            "range": "± 62.06",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_LRU",
            "value": 1106.65,
            "range": "± 4.88",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_enforcement_overhead_500episodes_RelevanceWeighted",
            "value": 32009.45,
            "range": "± 1306.95",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_1024",
            "value": 126421.82,
            "range": "± 364.19",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_256",
            "value": 32941.84,
            "range": "± 118.85",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_4096",
            "value": 511336.99,
            "range": "± 5743.4",
            "unit": "ns/iter"
          },
          {
            "name": "capacity_stress_capacity_64",
            "value": 8383.59,
            "range": "± 36.57",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_baseline_no_phase2",
            "value": 25057038.08,
            "range": "± 23047847.83",
            "unit": "ns/iter"
          },
          {
            "name": "combined_premem_genesis_overhead_genesis_only_summarization",
            "value": 8326013.21,
            "range": "± 1970769.42",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_with_compression_1",
            "value": 6.01,
            "range": "± 0.02",
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
            "name": "compression_overhead_without_compression_1",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_10",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_100",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_5",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compression_overhead_without_compression_50",
            "value": 0.27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_access_4_threads",
            "value": 105103.28,
            "range": "± 1316.69",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations",
            "value": 57719.72,
            "range": "± 579.81",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_1_read_latest@1",
            "value": 5377528427,
            "range": "± 989370707.02",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_latest_concurrency_4_read_latest@4",
            "value": 4800833311.2,
            "range": "± 794941148.27",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_16_read_mostly@16",
            "value": 4965957787.2,
            "range": "± 471868641.75",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_1_read_mostly@1",
            "value": 5510830451,
            "range": "± 820716682.92",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_4_read_mostly@4",
            "value": 4765960124.2,
            "range": "± 842994963.43",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_mostly_concurrency_8_read_mostly@8",
            "value": 5852342753.6,
            "range": "± 2551124054.44",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_16_read_only@16",
            "value": 5383305490.4,
            "range": "± 1003518712.85",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_1_read_only@1",
            "value": 4411258011.2,
            "range": "± 664994458.47",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_4_read_only@4",
            "value": 4848024771.2,
            "range": "± 1608653448.64",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_read_only_concurrency_8_read_only@8",
            "value": 5957745409.7,
            "range": "± 906739766.11",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_16_update_heavy@16",
            "value": 8388919699.2,
            "range": "± 1083815901.58",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_1_update_heavy@1",
            "value": 4618217439.5,
            "range": "± 394389181.6",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_4_update_heavy@4",
            "value": 5125265566,
            "range": "± 309616151.27",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent_operations_update_heavy_concurrency_8_update_heavy@8",
            "value": 6823273337.3,
            "range": "± 941640375.9",
            "unit": "ns/iter"
          },
          {
            "name": "connection_overhead_basic_pool",
            "value": 27693.39,
            "range": "± 99.72",
            "unit": "ns/iter"
          },
          {
            "name": "data_filtering",
            "value": 601834.56,
            "range": "± 1018.73",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_100",
            "value": 14181.27,
            "range": "± 1585.16",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_300",
            "value": 42045.01,
            "range": "± 4563.01",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_600",
            "value": 82017.06,
            "range": "± 6224.01",
            "unit": "ns/iter"
          },
          {
            "name": "domain_invalidation_latency_900",
            "value": 126544.15,
            "range": "± 9752.85",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_100",
            "value": 268,
            "range": "± 0.87",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_1000",
            "value": 2174.71,
            "range": "± 2.05",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_LRU_500",
            "value": 1111.43,
            "range": "± 4.99",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_100",
            "value": 6447.02,
            "range": "± 8.84",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_1000",
            "value": 63205.39,
            "range": "± 280.61",
            "unit": "ns/iter"
          },
          {
            "name": "eviction_algorithm_performance_RelevanceWeighted_500",
            "value": 31604.82,
            "range": "± 79.03",
            "unit": "ns/iter"
          },
          {
            "name": "hashmap_storage",
            "value": 18925.46,
            "range": "± 208.72",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_all_300_entries",
            "value": 32406.35,
            "range": "± 2439.3",
            "unit": "ns/iter"
          },
          {
            "name": "invalidation_comparison_invalidate_domain_100_entries",
            "value": 42820.44,
            "range": "± 7018.24",
            "unit": "ns/iter"
          },
          {
            "name": "lifecycle_simulation_episode_lifecycle_events",
            "value": 1562.45,
            "range": "± 29.12",
            "unit": "ns/iter"
          },
          {
            "name": "metrics_collection",
            "value": 5.45,
            "range": "± 0.03",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_10",
            "value": 819.01,
            "range": "± 10.68",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_20",
            "value": 829.68,
            "range": "± 6.72",
            "unit": "ns/iter"
          },
          {
            "name": "phase3_retrieval_accuracy_hierarchical_retrieval_5",
            "value": 764.72,
            "range": "± 5.66",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_with_domain",
            "value": 516.3,
            "range": "± 12.17",
            "unit": "ns/iter"
          },
          {
            "name": "put_overhead_without_domain",
            "value": 414.78,
            "range": "± 5.25",
            "unit": "ns/iter"
          },
          {
            "name": "redb_episode_retrieval",
            "value": 6058640.56,
            "range": "± 3934949.27",
            "unit": "ns/iter"
          },
          {
            "name": "redb_storage_init",
            "value": 5804775.86,
            "range": "± 8795874.4",
            "unit": "ns/iter"
          },
          {
            "name": "regex_pattern_matching",
            "value": 14854.75,
            "range": "± 98.19",
            "unit": "ns/iter"
          },
          {
            "name": "retrieval_accuracy_metrics_accuracy_web_api_query",
            "value": 787.62,
            "range": "± 8.31",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n10000_k1000",
            "value": 217136.77,
            "range": "± 240045.5",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n1000_k100",
            "value": 19712.46,
            "range": "± 7936.34",
            "unit": "ns/iter"
          },
          {
            "name": "select_top_k_crate_crate_n100_k10",
            "value": 1172.75,
            "range": "± 22.22",
            "unit": "ns/iter"
          },
          {
            "name": "send_event",
            "value": 27.23,
            "range": "± 0.2",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_20",
            "value": 22747.05,
            "range": "± 293.05",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_5",
            "value": 10376.76,
            "range": "± 102.17",
            "unit": "ns/iter"
          },
          {
            "name": "storage_compression_ratio_50",
            "value": 47811.55,
            "range": "± 564.19",
            "unit": "ns/iter"
          },
          {
            "name": "subscribe",
            "value": 27.27,
            "range": "± 0.1",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_20",
            "value": 6091.23,
            "range": "± 111.4",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_5",
            "value": 4016.77,
            "range": "± 83.48",
            "unit": "ns/iter"
          },
          {
            "name": "summary_generation_time_50",
            "value": 11029.77,
            "range": "± 49.53",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_1000",
            "value": 2847.36,
            "range": "± 42.74",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_10000",
            "value": 27028.36,
            "range": "± 377.54",
            "unit": "ns/iter"
          },
          {
            "name": "text_analysis_by_size_100000",
            "value": 251787.6,
            "range": "± 1417.49",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100000_k10000",
            "value": 2591912.22,
            "range": "± 34883.47",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n10000_k1000",
            "value": 165325.3,
            "range": "± 2607.78",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n1000_k100",
            "value": 12833.7,
            "range": "± 53.64",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_full_sort_n100_k10",
            "value": 1059.03,
            "range": "± 7.87",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100000_k10000",
            "value": 294047.92,
            "range": "± 4083.59",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n10000_k1000",
            "value": 28549.81,
            "range": "± 522.72",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n1000_k100",
            "value": 2410.08,
            "range": "± 10.66",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_selection_partial_sort_n100_k10",
            "value": 212.32,
            "range": "± 0.92",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k10",
            "value": 164805.04,
            "range": "± 824.35",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k100",
            "value": 164642.61,
            "range": "± 502.21",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k1000",
            "value": 165557.75,
            "range": "± 1774.05",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_full_n10000_k5000",
            "value": 165642.55,
            "range": "± 743.29",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k10",
            "value": 16167.72,
            "range": "± 92.11",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k100",
            "value": 17051.48,
            "range": "± 94.54",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k1000",
            "value": 28732.93,
            "range": "± 612.77",
            "unit": "ns/iter"
          },
          {
            "name": "top_k_varying_k_partial_n10000_k5000",
            "value": 93366.63,
            "range": "± 452.48",
            "unit": "ns/iter"
          },
          {
            "name": "vector_storage",
            "value": 39363.02,
            "range": "± 183.55",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}