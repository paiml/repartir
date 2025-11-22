# Repartir: Iron Lotus Framework Makefile
# Certeza Three-Tiered Testing Methodology

.PHONY: help tier1 tier2 tier3 test coverage clippy fmt check docs clean

help:
	@echo "Repartir - Sovereign AI Distributed Computing"
	@echo ""
	@echo "Iron Lotus Framework / Certeza Tiered Testing:"
	@echo "  tier1        - Quick checks (sub-second, for flow state)"
	@echo "  tier2        - Commit gates (1-5 min, comprehensive)"
	@echo "  tier3        - Merge gates (hours, mutation + formal)"
	@echo ""
	@echo "Individual targets:"
	@echo "  test         - Run all tests"
	@echo "  coverage     - Generate coverage report (≥95% target)"
	@echo "  clippy       - Run clippy linter"
	@echo "  fmt          - Format code"
	@echo "  check        - Cargo check"
	@echo "  docs         - Generate documentation"
	@echo "  audit        - Security audit (cargo-audit + cargo-deny)"
	@echo "  mutation     - Mutation testing (≥80% score target)"
	@echo "  clean        - Clean build artifacts"

# Tier 1: ON-SAVE (Sub-Second Feedback)
tier1:
	@echo "🔍 Tier 1: Quick Checks (Flow State)"
	cargo check --all-features
	cargo clippy --all-features --no-deps -- -D warnings
	cargo fmt -- --check
	cargo test --lib --all-features

# Tier 2: ON-COMMIT (1-5 Minute Gate)
tier2: tier1
	@echo "🔬 Tier 2: Commit Gates (Comprehensive)"
	cargo test --all-features
	cargo test --doc
	cargo llvm-cov --all-features --lcov --output-path lcov.info
	@echo "📊 Coverage Report:"
	cargo llvm-cov report --summary-only
	cargo doc --no-deps --all-features --document-private-items
	cargo audit
	cargo deny check

# Tier 3: ON-MERGE/NIGHTLY (Hours)
tier3: tier2
	@echo "⚗️  Tier 3: Merge Gates (Exhaustive)"
	cargo mutants --no-times
	@echo "🎯 Mutation Score Target: ≥80%"

# Individual targets
test:
	cargo test --all-features

coverage:
	cargo llvm-cov --all-features --html
	@echo "📊 Coverage report generated in target/llvm-cov/html/index.html"

clippy:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt

check:
	cargo check --all-features

docs:
	cargo doc --no-deps --all-features --document-private-items --open

audit:
	cargo audit
	cargo deny check

mutation:
	cargo mutants --no-times

clean:
	cargo clean
