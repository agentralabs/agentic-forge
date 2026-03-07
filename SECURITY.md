# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in AgenticForge, please report it
responsibly:

1. **Do not** open a public issue.
2. Email security@agentralabs.tech with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
3. You will receive an acknowledgment within 48 hours.
4. A fix will be developed and released before public disclosure.

## Security Practices

- No `.unwrap()` in MCP server code paths
- All FFI functions perform null checks before dereferencing
- Blueprint IDs use UUID v4 (cryptographically random)
- No network access by default (stdio transport only)
- No file system writes unless explicitly requested via export
- Input validation on all MCP tool parameters
- No secrets or credentials stored in blueprint data
