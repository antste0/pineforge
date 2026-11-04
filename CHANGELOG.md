# Changelog

## 0.2.0

- Only JPEG, PNG and WebP are supported now for both input and output, dropping the other formats (GIF, BMP, TIFF, AVIF and others) cuts compile time
- Changed the default value of `--strength` to `0.7`
- Added more examples to README
- Colors are now computed once per unique color through a lookup table instead of for every individual pixel, this speeds up the default mode and `--no-dither`

Total CPU time on a 27.28 MP photo, ThinkPad with AMD Ryzen 5 PRO 2500U.

| mode | 0.1.1 | 0.2.0 |
|---|---|---|
| `--all` | 80.38s | 10.05s |
| single palette | 26.90s | 3.56s |
| `--no-dither` | 9.03s | 3.43s |
| `--no-soft` (no change in code) | 8.64s | 8.61s |
