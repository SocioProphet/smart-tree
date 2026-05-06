.PHONY: validate prophet-understand-smoke

validate: prophet-understand-smoke
	@echo "OK: smart-tree validate"

prophet-understand-smoke:
	python3 tools/smoke_prophet_understanding_emitter.py
