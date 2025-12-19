# Javy v8.0.0 Research Findings

**Created**: 2025-12-14  
**Researcher**: web-search-researcher  
**Purpose**: Phase 2C Javy integration for memory-mcp

## Executive Summary

Javy v8.0.0 is a JavaScript to WebAssembly toolchain that provides configurable JavaScript runtime using QuickJS. Key findings:

- **Version Clarification**: Latest crate is javy 6.0.0 (CLI is v8.0.0)
- **Breaking Changes**: RQuickJS update broke bytecode compatibility in v8.0.0
- **Two Approaches**: Static linking (869KB+) vs Dynamic linking (1-16KB + plugin)
- **WASI Support**: Both preview 1 and 2 supported
- **Performance**: Moderate compilation overhead, good runtime performance

## Integration Strategy

### Recommended Approach: Dynamic Linking

**Reasons**:
- Smaller binary size (1-16KB vs 869KB+)
- Faster startup time
- Better for our use case (compiling JS on-demand)

### Dependencies Required

```toml
[dependencies]
javy = "6.0.0"
javy-codegen = "3.0.0"
javy-plugin-api = "3.1.0"
wasmtime = "24.0.5"
wasmtime-wasi = "24.0.5"
anyhow = "1.0"
```

### Compilation Workflow

1. **JavaScript Input**: Raw JS code string
2. **QuickJS Parse**: Convert to bytecode
3. **WASM Generation**: Embed bytecode in WASM module
4. **WASI Execution**: Run with stdio capture

## Key Implementation Details

### Core API Pattern

```rust
use javy_codegen::{Generator, LinkingKind, Plugin, JS};
use wasmtime::*;
use wasmtime_wasi::*;

pub struct JavyCompiler {
    engine: Engine,
    plugin: Plugin,
    linker: Linker<WasiCtx>,
}

impl JavyCompiler {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let plugin = Plugin::new_from_data(JAVY_PLUGIN_DATA)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        
        Ok(Self { engine, plugin, linker })
    }
    
    pub fn compile(&self, js_code: &str) -> Result<Vec<u8>> {
        let js = JS::from(js_code)?;
        let mut generator = Generator::new(self.plugin.clone());
        generator.linking(LinkingKind::Dynamic);
        
        let wasm = generator.generate(&js)?;
        Ok(wasm)
    }
}
```

### WASI Stdio Capture

```rust
use std::sync::{Arc, Mutex};
use wasmtime_wasi::{Stdout, Stderr, WasiCtxBuilder};

let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

let wasi = WasiCtxBuilder::new()
    .stdout(Stdout::from_pipe(Box::new(CapturePipe::new(stdout_buffer.clone()))))
    .stderr(Stderr::from_pipe(Box::new(CapturePipe::new(stderr_buffer.clone()))))
    .build();
```

## Limitations and Gotchas

### JavaScript Feature Limitations
- No async/await support (QuickJS limitation)
- Limited ES6+ features
- No DOM/Browser APIs
- Interpreter-only (no JIT)

### Integration Challenges
- **Plugin Management**: Need to bundle Javy plugin binary
- **Error Handling**: Complex JS error conversion required
- **Memory Management**: WASM memory constraints
- **Version Compatibility**: Breaking changes in v8.0.0

### Performance Considerations
- **Compilation Overhead**: JS → bytecode → WASM conversion
- **Binary Size**: Plugin binary adds to total size
- **Startup Time**: Dynamic linking faster than static

## Implementation Plan

### Phase 2C.1: Core Compiler Module
- Create `javy_compiler.rs` with compilation logic
- Bundle Javy plugin as binary data
- Implement error handling

### Phase 2C.2: WASI Enhancement
- Add stdout/stderr capture pipes
- Update `ExecutionResult` structure
- Integrate with existing wasmtime sandbox

### Phase 2C.3: UnifiedSandbox Integration
- Route JavaScript to Javy compiler
- Maintain fallback to Node.js
- Update test configuration

## Risk Assessment

### High Risk
- **Plugin Compatibility**: Version matching critical
- **Memory Usage**: WASM memory constraints

### Medium Risk
- **Performance**: Compilation overhead acceptable?
- **Error Handling**: Complex error scenarios

### Low Risk
- **API Stability**: Javy API is mature
- **WASI Support**: Well-documented

## Success Criteria

1. ✅ JavaScript code compiles to WASM
2. ✅ Console.log output captured
3. ✅ Errors handled gracefully
4. ✅ Integration with UnifiedSandbox
5. ✅ Performance acceptable for our use case

## Next Steps

1. Implement core Javy compiler module
2. Add WASI stdio capture
3. Integrate with UnifiedSandbox
4. Create comprehensive test suite
5. Benchmark against rquickjs baseline

---

**Status**: Research complete, ready for implementation  
**Confidence**: High - Javy v8.0.0 is well-documented and mature  
**Timeline Estimate**: 2-3 sessions for full integration