#!/usr/bin/env make

ifeq ($(GIT_ROOT),)
	GIT_ROOT:=$(shell git rev-parse --show-toplevel)
endif

all: format lint vet build test

build:
	${GIT_ROOT}/make/build

clean:
	${GIT_ROOT}/make/clean

lint:
	${GIT_ROOT}/make/lint

format:
	${GIT_ROOT}/make/format

vet:
	${GIT_ROOT}/make/vet

test:
	${GIT_ROOT}/make/test

swagger-validate:
	${GIT_ROOT}/make/swagger-validate

swagger-generate: swagger-validate
	${GIT_ROOT}/make/swagger-generate

mock-generate:
	${GIT_ROOT}/make/mock-generate

.PHONY: clean format lint vet build test
