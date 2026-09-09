window.BENCHMARK_DATA = {
  "lastUpdate": 1788965865929,
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
      }
    ]
  }
}