# Security Review

You are performing a security review of a CLI application. Focus on practical vulnerabilities, not theoretical concerns. This is a local CLI tool that manages issues/tasks, so prioritize relevant attack vectors.

## Review Areas

### 1. Input Handling

- [ ] **Command line arguments** - Injection risks? Shell escapes?
- [ ] **Title/note content** - Stored safely? Special characters handled?
- [ ] **File paths** - Path traversal possible? Symlink attacks?
- [ ] **Tag parsing** - Any special characters cause issues?
- [ ] **ID parsing** - Integer overflow? Format string attacks?

### 2. File System

- [ ] **Config file parsing** - TOML/JSON injection? Arbitrary code execution?
- [ ] **Database path** - Can user control location unsafely?
- [ ] **Export paths** - Writing to sensitive locations?
- [ ] **Temp files** - Secure creation? Race conditions? Cleaned up?
- [ ] **Permissions** - Files created with appropriate permissions?

### 3. SQL/Database

- [ ] **Parameterized queries** - Used consistently?
- [ ] **String concatenation** - Any dynamic SQL construction?
- [ ] **SQLite permissions** - Database file permissions appropriate?
- [ ] **SQL injection** - Any user input reaching SQL?

### 4. Dependencies

Run and report results of:
- **Rust:** `cargo audit`
- **Go:** `govulncheck ./...`
- **TypeScript:** `npm audit` or `bun audit`

Check for:
- [ ] Known CVEs in dependencies?
- [ ] Minimal dependency surface?
- [ ] Dependencies from trusted sources?
- [ ] Outdated dependencies with known issues?

### 5. Error Handling

- [ ] **Sensitive info leaked in errors?** - Paths, internal state, etc.
- [ ] **Graceful failure vs panics?** - Crashes on bad input?
- [ ] **Error messages safe for display?** - No format strings, etc.

### 6. Secrets/Credentials

- [ ] **Hardcoded secrets?** - API keys, passwords, tokens
- [ ] **Credentials in config?** - Handled safely if present
- [ ] **Environment variables?** - Secure reading, no logging

### 7. Logic Issues

- [ ] **Race conditions** - TOCTOU issues with file operations?
- [ ] **Denial of service** - Large inputs cause hangs/memory exhaustion?
- [ ] **Resource leaks** - File handles, memory not freed?

## Output Format

---

## [Language] Security Review

**Risk Level: Low / Medium / High**

### Vulnerabilities Found

1. **[SEVERITY: Critical/High/Medium/Low]** Title
   - **File:** path:line
   - **Description:** What the issue is
   - **Impact:** What could happen if exploited
   - **Fix:** How to remediate

### Dependency Audit

```
[Paste output of cargo audit / govulncheck / npm audit]
```

- [List any known CVEs or concerns]
- [Note any dependencies that should be updated]

### Positive Findings

- [Security measures done well]
- [Good practices observed]

### Recommendations

1. **[Priority: High]** Description
2. **[Priority: Medium]** Description
3. **[Priority: Low]** Description

---

## Severity Guide

- **Critical:** Remote code execution, authentication bypass, data corruption
- **High:** Local code execution, sensitive data exposure, SQLi
- **Medium:** Information disclosure, DoS, privilege escalation
- **Low:** Minor info leak, poor practice, defense-in-depth suggestions

## Review Checklist

Before submitting review, verify:

- [ ] Ran dependency audit tools
- [ ] Checked all user input entry points
- [ ] Reviewed file and database operations
- [ ] Looked for hardcoded secrets
- [ ] Considered the threat model (local CLI tool)
- [ ] Recommendations are actionable and prioritized

## Threat Model Notes

This is a **local CLI tool** that:
- Runs with user's permissions
- Reads/writes local files only
- Has no network functionality (unless syncing)
- Manages non-sensitive data (issue tracking)

Focus on:
- Input validation (malicious issue titles, paths)
- File system safety (path traversal, permissions)
- Data integrity (SQL injection, corruption)

Lower priority for this context:
- Network security (unless sync feature exists)
- Authentication (local user is already trusted)
- Encryption at rest (local data, user's disk)
