## xrat v0.30.2

Enable Thin Link-Time Optimization (LTO) and symbol stripping for release builds,
while keeping local development builds unoptimized for fast iteration.

### Build improvements

- Release builds use `opt-level = 3` with Thin LTO for cross-crate optimization.
- Release binaries strip symbols to reduce distributed executable size.
- Development builds explicitly use `opt-level = 0`; incremental compilation and
  debugging information keep their Cargo defaults.
- Existing release and Docker build commands automatically use these settings.

### Upgrade notes

- No CLI, configuration, or database schema changes.
- Release builds can take longer due to Thin LTO. Runtime performance and size
  improvements vary by target and workload; no benchmark speedup is claimed.
- Stripped release binaries provide less symbol information for diagnostics.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.30.1...v0.30.2
