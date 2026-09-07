window.BENCHMARK_DATA = {
  "lastUpdate": 1788749605782,
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
      }
    ]
  }
}