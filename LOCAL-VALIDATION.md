# Local validation

HTP does **not** use GitHub Actions. Validation is run locally or on an operator-controlled machine.

Normal HTP gate:

```bash
bash scripts/validate-local.sh
```

This reproduces the former reference implementation, schema/browser, local-only runtime, architecture-boundary, and public example checks.

Optional Rust 1.78 shipped-target gate:

```bash
bash scripts/validate-local.sh --msrv
```

Run everything, including MSRV:

```bash
bash scripts/validate-local.sh --all
```

The MSRV mode requires Rust 1.78.0 to already be installed. Historical Actions definitions remain recoverable through Git history, but active `.github/workflows` is intentionally absent.
