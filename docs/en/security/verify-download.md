# Verify Download (EN)

- Trust root SoT: `contracts/security/v1.0/signing_trust_root.v1.json`
- Current key ID: `w18-sot-root`
- Required verification order:
  1. `release_artifact_manifest.json.sig`
  2. `SHA256SUMS.sig`
- Do not fall back if the primary manifest/signature pair mismatches.
- `GitHub auto-generated source archives` are outside the default trust chain unless they are explicitly listed as official signed assets.
- The current installer uses `code_signing_mode=none`, so Windows SmartScreen warnings are expected. Always verify manifest + checksums + signatures before running a release artifact.

Quick rule:
- trial use: you may inspect the package first;
- production or trust-sensitive use: verify manifest, checksums, and signatures before execution.

Minimum verification flow:
1. Verify the trust root.
2. Verify `release_artifact_manifest.json.sig`.
3. Verify `SHA256SUMS.sig`.
4. Compare the artifact hash against `SHA256SUMS`.
5. Stop immediately on any mismatch.

Verification precedence:
- primary source of truth: `release_artifact_manifest.json` + `release_artifact_manifest.sig`
- secondary source: `SHA256SUMS` + `SHA256SUMS.sig`
- if the primary pair fails, do not continue with the secondary pair as a fallback

Windows note:
- because the installer is not Authenticode-signed at this stage, SmartScreen may warn;
- that warning does not replace cryptographic verification;
- the supported trust path is still manifest + checksum + signature verification.
