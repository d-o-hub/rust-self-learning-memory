========================================
MCP Server Reconnection Test Results
Date: Wed Dec 10 04:57:56 PM UTC 2025
Server: ./target/release/memory-mcp-server
========================================

[0;34m[TEST 1] Single Request (Baseline)[0m
Testing single request to ensure server works...
Duration: 102ms
Status: PASSED

[0;34m[TEST 2] Multiple Sequential Requests (5 iterations)[0m
Testing 5 sequential requests from same client...
=== Iteration 1 ===
=== Iteration 2 ===
=== Iteration 3 ===
=== Iteration 4 ===
=== Iteration 5 ===
Total Duration: 3210ms
Status: CHECK /tmp/test2_output.txt for results

[0;34m[TEST 3] Multiple Sequential Requests (10 iterations)[0m
Testing 10 sequential requests from same client...
=== Iteration 1 ===
=== Iteration 2 ===
=== Iteration 3 ===
=== Iteration 4 ===
=== Iteration 5 ===
=== Iteration 6 ===
=== Iteration 7 ===
=== Iteration 8 ===
=== Iteration 9 ===
=== Iteration 10 ===
Total Duration: 7295ms
Status: CHECK /tmp/test3_output.txt for results

[0;34m[TEST 4] Continuous Stream (20 requests without delay)[0m
Testing continuous stream of 20 requests...
Duration: 204ms
Status: CHECK /tmp/test4_output.txt for results

[0;34m[TEST 5] Reconnection after Brief Pause (1 second)[0m
Testing reconnection after 1 second pause...
Duration: 1092ms
Status: CHECK /tmp/test5_output.txt for results

[0;34m[TEST 6] Reconnection after Long Pause (5 seconds)[0m
Testing reconnection after 5 second pause...
Duration: 5061ms
Status: CHECK /tmp/test6_output.txt for results

[0;34m[TEST 7] Rapid Reconnection (0.1 second delays, 15 iterations)[0m
Testing rapid reconnection with 0.1s delays...
=== Iteration 1 ===
=== Iteration 2 ===
=== Iteration 3 ===
=== Iteration 4 ===
=== Iteration 5 ===
=== Iteration 6 ===
=== Iteration 7 ===
=== Iteration 8 ===
=== Iteration 9 ===
=== Iteration 10 ===
=== Iteration 11 ===
=== Iteration 12 ===
=== Iteration 13 ===
=== Iteration 14 ===
=== Iteration 15 ===
Total Duration: 3598ms
Status: CHECK /tmp/test7_output.txt for results

[0;34m[TEST 8] Simulated Client Session with Tool Calls[0m
Testing complete client session with various tool calls...
Duration: 4071ms
Status: CHECK /tmp/test8_output.txt for results

========================================
Test Summary
========================================
All tests completed. Check output files:
  - /tmp/test1_output.txt - Single request
  - /tmp/test2_output.txt - 5 sequential requests
  - /tmp/test3_output.txt - 10 sequential requests
  - /tmp/test4_output.txt - 20 continuous requests
  - /tmp/test5_output.txt - Reconnection after 1s pause
  - /tmp/test6_output.txt - Reconnection after 5s pause
  - /tmp/test7_output.txt - Rapid reconnection (0.1s delays)
  - /tmp/test8_output.txt - Complete client session

Results saved to: ./plans/interaction_test_results.md
========================================
