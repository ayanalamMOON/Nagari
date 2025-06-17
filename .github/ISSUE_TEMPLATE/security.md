---
name: Security Vulnerability
about: Report a security vulnerability (use only for public, non-critical issues)
title: '[SECURITY] '
labels: ['security', 'needs-triage']
assignees: ''

---

## 🔒 Security Issue

> ⚠️ **IMPORTANT**: For critical security vulnerabilities, please report privately to <security@nagari-lang.org> instead of creating a public issue.

**Severity Level:**

- [ ] Low - Minor security improvement
- [ ] Medium - Potential security concern
- [ ] High - Exploitable vulnerability (should be reported privately)
- [ ] Critical - Severe vulnerability (MUST be reported privately)

**Component Affected:**

- [ ] Compiler (nagari-compiler)
- [ ] Runtime/VM (nagari-vm)
- [ ] CLI Tools
- [ ] REPL
- [ ] LSP Server
- [ ] WebAssembly Runtime
- [ ] Embedded Runtime
- [ ] Package Manager
- [ ] Registry Server
- [ ] Build System

## 🎯 Vulnerability Description

**Summary:**
A clear and concise description of the security issue.

**Attack Vector:**
How could this vulnerability be exploited?

**Impact:**
What could an attacker achieve by exploiting this?

## 🔍 Technical Details

**Vulnerable Code/Component:**
Identify the specific code, function, or component affected.

**Root Cause:**
What is the underlying cause of this vulnerability?

**Prerequisites:**
What conditions must be met for this vulnerability to be exploitable?

## 📋 Proof of Concept

**Reproduction Steps:**

1. Step 1
2. Step 2
3. Step 3

**Minimal Example:**

```nagari
// Provide a minimal code example that demonstrates the vulnerability
// Make sure this is safe to share publicly
fn vulnerable_example() {
    // Your example here
}
```

**Expected vs Actual Behavior:**

- **Expected**: What should happen securely
- **Actual**: What actually happens (the vulnerability)

## 🌍 Environment

**System Information:**

- OS: [e.g. Windows 11, macOS 14, Ubuntu 22.04]
- Architecture: [e.g. x64, ARM64]
- Nagari Version: [e.g. 0.2.1]

**Additional Context:**

- Runtime environment (Node.js, browser, etc.)
- Any specific configuration or setup

## 🛡️ Suggested Mitigation

**Immediate Workarounds:**

What can users do to protect themselves while waiting for a fix?

**Proposed Solution:**

How do you think this vulnerability should be fixed?

**Security Best Practices:**

Any general recommendations for preventing similar issues?

## 📊 Impact Assessment

**Affected Users:**

- [ ] All users
- [ ] Users with specific configurations
- [ ] Developers only
- [ ] Server deployments only
- [ ] Limited scope: ___________

**Exploitation Difficulty:**

- [ ] Trivial - Can be exploited easily
- [ ] Easy - Requires basic knowledge
- [ ] Moderate - Requires specific conditions
- [ ] Hard - Requires advanced techniques

**Potential Damage:**

- [ ] Information disclosure
- [ ] Code execution
- [ ] Privilege escalation
- [ ] Denial of service
- [ ] Data corruption
- [ ] Other: ___________

## ✅ Checklist

- [ ] I have confirmed this is a security issue
- [ ] I have assessed this as low/medium severity (high/critical issues should be reported privately)
- [ ] I have provided sufficient details for reproduction
- [ ] I have considered the impact on users
- [ ] I have suggested possible mitigations

## 🔗 References

**Related CVEs:**
Link to any related Common Vulnerabilities and Exposures.

**Similar Issues:**
Link to similar security issues in other projects.

**Security Research:**
Link to relevant security research or documentation.

## 📝 Responsible Disclosure

**Timeline:**

- Discovery Date: [When did you discover this?]
- Notification Date: [When are you reporting this?]
- Proposed Disclosure: [When should this be made public?]

**Credit:**

- Would you like to be credited in the security advisory?
- How should you be credited? (Name, handle, organization)

---

**Remember**: For critical vulnerabilities that could cause immediate harm, please email <security@nagari-lang.org> instead of creating a public issue.
