# FFI C API Reference

The `agentic-forge-ffi` crate exposes a C-compatible API for embedding
AgenticForge in non-Rust applications.

## Functions

### agentic_forge_version

```c
const char* agentic_forge_version(void);
```

Returns the version string. The returned pointer is static and must not be freed.

### agentic_forge_create_blueprint

```c
char* agentic_forge_create_blueprint(
    const char* name,
    const char* description,
    const char* domain
);
```

Creates a blueprint and returns the blueprint ID as a heap-allocated string.
Returns NULL on failure. The caller must free the result with
`agentic_forge_free_string`.

### agentic_forge_free_string

```c
void agentic_forge_free_string(char* s);
```

Frees a string allocated by AgenticForge. Safe to call with NULL.

### agentic_forge_invention_count

```c
uint32_t agentic_forge_invention_count(void);
```

Returns the number of inventions (32).

### agentic_forge_tool_count

```c
uint32_t agentic_forge_tool_count(void);
```

Returns the number of MCP tools (15).

## Usage from C

```c
#include <stdio.h>

extern const char* agentic_forge_version(void);
extern char* agentic_forge_create_blueprint(const char*, const char*, const char*);
extern void agentic_forge_free_string(char*);

int main() {
    printf("Version: %s\n", agentic_forge_version());
    char* id = agentic_forge_create_blueprint("test", "A test project", "api");
    if (id) {
        printf("Blueprint ID: %s\n", id);
        agentic_forge_free_string(id);
    }
    return 0;
}
```

## Linking

```bash
cargo build --release -p agentic-forge-ffi
# Link against target/release/libagentic_forge_ffi.a (static)
# Or target/release/libagentic_forge_ffi.dylib (dynamic)
```
