# Code Idiomaticness Review

You are reviewing a CLI application implementation for idiomaticness - how well it follows the conventions and best practices of its language ecosystem.

## Review Criteria

For each file, assess:

### 1. Naming Conventions
- Variables, functions, types follow language style guide
- **Rust:** snake_case functions, CamelCase types, SCREAMING_SNAKE_CASE constants
- **Go:** CamelCase exported, camelCase private, avoid stuttering (pkg.PkgThing)
- **TypeScript:** camelCase functions, PascalCase types/classes/interfaces

### 2. Error Handling
- **Rust:** Uses Result<T, E>, ? operator, thiserror/anyhow appropriately, no unwrap() in library code
- **Go:** Checks errors immediately, wraps with context (fmt.Errorf or errors.Wrap), returns early on error
- **TypeScript:** Uses typed errors, async/await properly, never catches and ignores

### 3. Project Structure
- Follows standard layout for the language
- **Rust:** src/lib.rs, src/main.rs, logical modules, pub only what's needed
- **Go:** cmd/, internal/, pkg/ or flat structure, package names match directory
- **TypeScript:** src/, index.ts exports, types in separate files or co-located

### 4. API Design
- Functions/methods have clear, single purposes
- Appropriate use of language features (traits, interfaces, generics)
- Not fighting the language (e.g., OOP in Go, mutable patterns in Rust)
- Parameters are appropriately typed (not stringly-typed)

### 5. Dependencies
- Uses standard library where appropriate
- External deps are well-chosen, not excessive
- No reinventing wheels unnecessarily
- Dependencies are maintained and secure

### 6. Common Patterns
- Builder pattern, iterators, etc. used idiomatically
- No anti-patterns:
  - God objects
  - Stringly-typed interfaces
  - Inheritance misuse
  - Global mutable state
  - Callback hell

### 7. Documentation
- Public APIs documented appropriately
- Complex logic has explanatory comments
- No obvious/redundant comments

## Output Format

For each implementation, provide:

---

## [Language] Implementation

**Idiomaticness Score: X/10**

### Strengths
- [List 3-5 things done well with file:line references]

### Issues
- [List specific non-idiomatic patterns with file:line references]
- [Include idiomatic alternative for each issue]

### Recommendations
1. [High priority] Description
2. [Medium priority] Description
3. [Low priority] Description

---

## Scoring Guide

- **9-10:** Exemplary. Could be used as teaching material.
- **7-8:** Good. Follows conventions with minor deviations.
- **5-6:** Average. Some non-idiomatic patterns that should be addressed.
- **3-4:** Below average. Significant idiom violations.
- **1-2:** Poor. Appears written by someone unfamiliar with the language.

## Review Checklist

Before submitting review, verify:

- [ ] Reviewed all source files (not just entry point)
- [ ] Considered language-specific idioms, not general programming style
- [ ] Recommendations are actionable and specific
- [ ] Score reflects actual code quality, not personal preferences
