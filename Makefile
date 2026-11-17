.DEFAULT_GOAL := build

CARGO ?= cargo
PREFIX ?= $(HOME)/.local
DESTDIR ?=
CARGO_TARGET_DIR ?= target

.PHONY: build install

build:
	$(CARGO) build --release --locked --target-dir "$(CARGO_TARGET_DIR)"

install: build
	install -Dm755 "$(CARGO_TARGET_DIR)/release/umbrellascreen" "$(DESTDIR)$(PREFIX)/bin/umbrellascreen"
