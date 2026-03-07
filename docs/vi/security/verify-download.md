# Verify Download (VI)

- Trust root SoT: `contracts/security/v1.0/signing_trust_root.v1.json`
- Key ID hiện hành: `w18-sot-root`
- Thứ tự verify bắt buộc:
  1. `release_artifact_manifest.json.sig`
  2. `SHA256SUMS.sig`
- Không được fallback nếu manifest/signature chính bị lệch.
- `GitHub auto-generated source archives` không nằm trong trust chain mặc định; chỉ coi là trusted nếu sau này được liệt kê rõ như signed asset chính thức.
- Installer hiện tại dùng `code_signing_mode=none`, nên Windows SmartScreen có thể cảnh báo. Luôn đối chiếu manifest + checksum + signature trước khi chạy bản tải.
