# Changelog

## 0.2.0

- Only JPEG, PNG and WebP are supported now, dropping the other formats cuts compile time
- Changed the default value of `--strength` to `0.7`
- Added more examples to README
- The LUT now computes each unique color once instead of every pixel separately

Total CPU time on a 27.28 MP photo, ThinkPad with AMD Ryzen 5 PRO 2500U.

| mode | 0.1.1 | 0.2.0 |
|---|---|---|
| `--all` | 75.58s | 5.83s |
| single palette | 26.72s | 3.51s |
| `--no-soft` (no change in code) | 8.53s | 8.42s |
