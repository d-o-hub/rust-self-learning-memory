# MCP Server Error Response Formats

## Overview

The Memory MCP server implements proper JSON-RPC 2.0 error handling to ensure clients always receive valid responses, even when internal errors occur.

## Error Response Format

All error responses follow the JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "id": "<request_id>",
  "error": {
    "code": <error_code>,
    "message": "<error_message>",
    "data": {
      "details": "<detailed_error_description>"
    }
  }
}
```

## Error Codes

### Standard JSON-RPC Error Codes
- `-32700`: Parse error - Invalid JSON received
- `-32600`: Invalid request - The JSON sent is not a valid Request object
- `-32601`: Method not found - The method does not exist or is not available
- `-32602`: Invalid params - Invalid method parameter(s)
- `-32603`: Internal error - Internal JSON-RPC error

### MCP-Specific Error Codes
- `-32000`: Tool execution failed - A tool call failed during execution

## Error Scenarios

### 1. JSON Serialization Failures
When the server fails to serialize response data to JSON:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": {
      "details": "Response serialization failed: <specific_error>"
    }
  }
}
```

### 2. Invalid Method Calls
When a client requests a non-existent method:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": {
      "method": "unknown_method"
    }
  }
}
```

### 3. Tool Execution Failures
When a tool call fails during execution:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32000,
    "message": "Tool execution failed",
    "data": {
      "details": "<tool_specific_error>"
    }
  }
}
```

## Client Handling

Clients should always check for the presence of `error` vs `result` in responses:

```javascript
if (response.error) {
  // Handle error based on error.code
  switch (response.error.code) {
    case -32603:
      // Internal error - retry or report
      break;
    case -32601:
      // Method not found - check method name
      break;
    // ... handle other codes
  }
} else {
  // Process successful result
  processResult(response.result);
}
```

## Server Stability

The server is designed to never panic due to JSON serialization failures. All `serde_json::to_value()` calls are wrapped in proper error handling that returns structured JSON-RPC error responses instead of crashing.

## Troubleshooting

### Common Issues

1. **"failed parse server response"**: This should no longer occur as the server always returns valid JSON. If it happens, check client-side JSON parsing.

2. **Error code -32603**: Indicates internal server error. Check server logs for details in the `data.details` field.

3. **Malformed responses**: Should not occur. If they do, it's a bug in the server that needs immediate fixing.

### Debugging

Enable debug logging to see detailed error information:
```bash
RUST_LOG=debug cargo run --bin memory-mcp-server
```

Look for log messages like:
```
ERROR Failed to serialize initialize response: <error_details>
```

## Troubleshooting Guide

### Problem: "failed parse server response"

**Symptoms:**
- Client receives responses that can't be parsed as JSON
- JSON parsing errors in client applications
- Intermittent connection issues

**Root Causes:**
1. **Server panic** (should no longer occur after fix)
2. **Client-side JSON parsing issues**
3. **Network corruption**
4. **Memory corruption in server**

**Solutions:**
1. **Check server logs** for panic messages or serialization errors
2. **Verify client JSON parser** can handle valid JSON-RPC responses
3. **Test with simple requests** to isolate the issue
4. **Check network connectivity** and proxy configurations

### Problem: Error code -32603 (Internal error)

**Symptoms:**
- Server returns internal error responses
- Operations fail unexpectedly
- Logs show serialization failures

**Possible Causes:**
1. **Memory pressure** causing JSON allocation failures
2. **Complex data structures** that can't be serialized
3. **Custom types** without proper Serialize implementation
4. **Large response payloads** exceeding memory limits

**Solutions:**
1. **Check memory usage** and system resources
2. **Review data structures** being returned by tools
3. **Implement pagination** for large result sets
4. **Add response size limits** if needed

### Problem: Tool execution failures (-32000)

**Symptoms:**
- Tool calls return error responses
- Specific tool operations fail
- Error details in `data.details` field

**Common Issues:**
1. **Code execution errors** in sandbox
2. **Invalid tool parameters**
3. **Resource limits exceeded**
4. **Security violations**

**Solutions:**
1. **Check tool parameters** are correctly formatted
2. **Review sandbox logs** for execution details
3. **Verify resource limits** are appropriate
4. **Test tools individually** to isolate issues

### Problem: Method not found (-32601)

**Symptoms:**
- "Method not found" error responses
- Tool calls fail with unknown method

**Causes:**
1. **Typo in method name**
2. **Outdated client** calling deprecated methods
3. **Server configuration issues**

**Solutions:**
1. **Verify method names** match server capabilities
2. **Check server logs** for available tools
3. **Update client** to use correct method names

### Monitoring and Alerting

**Key Metrics to Monitor:**
- Error response rate (should be < 1%)
- Serialization failure count (should be 0)
- Tool execution success rate
- Response time percentiles

**Alert Conditions:**
- Error rate > 5% for 5 minutes
- Any serialization failures
- Tool success rate < 95%

### Performance Considerations

**Response Size Limits:**
- Keep responses under 1MB to prevent serialization issues
- Implement pagination for large datasets
- Compress responses if supported by client

**Timeout Handling:**
- Set appropriate timeouts for long-running operations
- Return timeout errors instead of hanging
- Implement request cancellation

### Best Practices

1. **Always check for errors** in client code before processing results
2. **Log error details** on both client and server for debugging
3. **Implement retry logic** for transient errors
4. **Monitor error rates** and alert on anomalies
5. **Keep error messages** descriptive but not verbose
6. **Use appropriate HTTP status codes** when applicable