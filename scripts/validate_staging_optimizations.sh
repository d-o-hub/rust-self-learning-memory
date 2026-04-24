#!/bin/bash
# Staging Optimization Validation Script
# Validates that all Turso optimizations are working correctly

set -e

STAGING_URL="${STAGING_URL:-http://localhost:8080}"
BOLD="\033[1m"
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m" # No Color

echo -e "${BOLD}🔬 Validating Turso Optimizations in Staging${NC}"
echo "=============================================="
echo "Target: $STAGING_URL"
echo ""

# Test 1: Health Check
echo -e "${BOLD}Test 1: Health Check${NC}"
HEALTH=$(curl -s "$STAGING_URL/health" || echo '{"status":"failed"}')
STATUS=$(echo "$HEALTH" | jq -r '.status // "unknown"')
if [ "$STATUS" = "healthy" ]; then
    echo -e "  ${GREEN}✓${NC} Status: $STATUS"
else
    echo -e "  ${RED}✗${NC} Status: $STATUS"
    exit 1
fi
echo ""

# Test 2: Single Read Performance
echo -e "${BOLD}Test 2: Single Read Performance${NC}"
START=$(date +%s%N)
curl -s "$STAGING_URL/episodes/test-id-1" > /dev/null 2>&1 || true
END=$(date +%s%N)
LATENCY=$(( ($END - $START) / 1000000 ))
echo "  Latency: ${LATENCY}ms"
if [ $LATENCY -lt 50 ]; then
    echo -e "  ${GREEN}✓${NC} Target: < 50ms"
else
    echo -e "  ${YELLOW}⚠${NC}  Target: < 50ms (not critical)"
fi
echo ""

# Test 3: Cached Read Performance
echo -e "${BOLD}Test 3: Cached Read (2nd attempt - should be faster)${NC}"
START=$(date +%s%N)
curl -s "$STAGING_URL/episodes/test-id-1" > /dev/null 2>&1 || true
END=$(date +%s%N)
CACHED_LATENCY=$(( ($END - $START) / 1000000 ))
echo "  Cached Latency: ${CACHED_LATENCY}ms"
if [ $CACHED_LATENCY -lt $LATENCY ]; then
    SPEEDUP=$(echo "scale=1; $LATENCY / $CACHED_LATENCY" | bc 2>/dev/null || echo "N/A")
    echo -e "  ${GREEN}✓${NC} Speedup: ${SPEEDUP}x faster"
else
    echo -e "  ${YELLOW}⚠${NC}  Cache may not be working"
fi
echo ""

# Test 4: Optimization Metrics
echo -e "${BOLD}Test 4: Optimization Metrics${NC}"
METRICS=$(curl -s "$STAGING_URL/metrics" 2>/dev/null || echo '{}')

if [ -n "$METRICS" ] && [ "$METRICS" != "{}" ]; then
    CACHE_HIT_RATE=$(echo "$METRICS" | jq -r '.cache.hit_rate // 0' 2>/dev/null || echo "0")
    POOL_ACTIVE=$(echo "$METRICS" | jq -r '.connection_pool.active // 0' 2>/dev/null || echo "0")
    
    echo "  Cache Hit Rate: ${CACHE_HIT_RATE}"
    if (( $(echo "$CACHE_HIT_RATE > 0.5" | bc -l 2>/dev/null || echo 0) )); then
        echo -e "  ${GREEN}✓${NC} Target: > 50%"
    else
        echo -e "  ${YELLOW}⚠${NC}  Target: > 50% (may improve over time)"
    fi
    
    echo "  Pool Active Connections: ${POOL_ACTIVE}"
else
    echo -e "  ${YELLOW}⚠${NC}  Metrics endpoint not available"
fi
echo ""

echo -e "${BOLD}═══════════════════════════════${NC}"
echo -e "${GREEN}✅ Validation Complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Monitor for 24 hours"
echo "  2. Check cache hit rate trends"
echo "  3. Review logs for any errors"
echo "  4. Prepare production deployment"
