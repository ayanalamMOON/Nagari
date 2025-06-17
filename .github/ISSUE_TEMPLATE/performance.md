---
name: Performance Issue
about: Report performance problems or regressions
title: '[PERFORMANCE] '
labels: ['performance', 'needs-investigation']
assignees: ''

---

## ⚡ Performance Issue

**Component Affected:**

- [ ] Compiler (nagari-compiler)
- [ ] Runtime/VM (nagari-vm)
- [ ] CLI Tools
- [ ] REPL
- [ ] LSP Server
- [ ] WebAssembly Runtime
- [ ] Embedded Runtime
- [ ] Package Manager
- [ ] Build System

**Performance Problem:**

- [ ] Slow compilation times
- [ ] High memory usage during compilation
- [ ] Slow runtime execution
- [ ] High memory usage during execution
- [ ] Slow startup time
- [ ] Poor scaling with input size
- [ ] Regression from previous version

## 📊 Performance Details

**Current Performance:**

- **Metric**: [e.g. compilation time, memory usage, execution time]
- **Measured Value**: [e.g. 5.2 seconds, 256MB, 150ms]
- **Expected Value**: [e.g. <2 seconds, <100MB, <50ms]

**Benchmark Information:**

```nagari
// Paste the code that demonstrates the performance issue
fn performance_test() {
    // Your test case here
}
```

**System Information:**

- **OS**: [e.g. Windows 11, macOS 14, Ubuntu 22.04]
- **CPU**: [e.g. Intel i7-12700K, Apple M2, AMD Ryzen 7]
- **RAM**: [e.g. 16GB, 32GB]
- **Storage**: [e.g. SSD, NVMe]

## 🔄 Reproduction

**Steps to Reproduce:**

1. Create file with the following content: `[file content]`
2. Run command: `[command]`
3. Measure: `[what to measure]`

**Reproducible Test Case:**

```bash
# Commands to reproduce the performance issue
time nagari compile large_file.nag
# Or
hyperfine "nagari run benchmark.nag"
```

## 📈 Benchmarks

**Performance Measurements:**

| Version | Compile Time | Memory Usage | Runtime |
|---------|--------------|--------------|---------|
| Current | [time]       | [memory]     | [time]  |
| Expected| [time]       | [memory]     | [time]  |

**Profiling Results:**

If you have profiling data, please include:

- CPU profiling results
- Memory profiling results
- Flamegraphs or call graphs
- Hot spots identified

## 🔍 Analysis

**Suspected Cause:**

What do you think might be causing the performance issue?

**Regression Information:**

- [ ] This is a new issue (not a regression)
- [ ] This worked better in version: ___________
- [ ] This has always been slow
- [ ] Unsure about regression status

**Impact Assessment:**

- [ ] Blocks development workflow
- [ ] Makes Nagari unsuitable for production
- [ ] Affects user experience significantly
- [ ] Minor inconvenience

## 🛠️ Additional Context

**Environment Variables:**

Any relevant environment variables or configuration?

**Hardware Constraints:**

Is this specific to certain hardware configurations?

**Comparison with Other Tools:**

How does this compare to similar operations in other languages/tools?

## ✅ Checklist

- [ ] I have provided a minimal reproducible case
- [ ] I have included performance measurements
- [ ] I have specified the affected component
- [ ] I have checked if this is a known issue
