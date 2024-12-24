# Security Policy

## Supported Versions

As Cal-rs is currently in early development (v0.1.0), we're maintaining security updates only for the latest version.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of Cal-RS seriously. If you discover a security vulnerability, please follow these steps:

1. **Do Not** create a public GitHub issue for the vulnerability.
2. Send an email to 22am014@sctce.ac.in with:
   - A description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Suggestions for fixing (if any)

### What to Expect

- You'll receive an acknowledgment within 48 hours
- We'll investigate and keep you updated on our findings
- Once fixed, we'll:
  - Release a patch
  - Credit you (unless you prefer to remain anonymous)
  - Document the vulnerability

## Current Security Measures

As the project is in its initial phase (referencing main.rs, lines 1-3), we currently have minimal attack surface. However, as we add features, we'll:

1. Regularly update dependencies
2. Implement secure file handling for calendar data
3. Follow Rust security best practices
4. Conduct code reviews focusing on security

## Future Security Enhancements

As the project grows, we plan to:
- Implement data encryption for stored calendar information
- Add authentication for calendar sharing features
- Establish secure sync protocols for calendar integration
- Set up automated security scanning

This security policy will be updated as the project evolves and new features are added. 