BINARY=unit-01
VERSION=0.1.0
BUILD_DIR=bin

.PHONY: all build run test clean install release

all: build

build:
	@echo "Building Orchestrator $(VERSION)..."
	@go build -o $(BINARY) .

run:
	@go run .

test:
	@echo "Running unit and integration tests..."
	@go test ./... -v

clean:
	@echo "Cleaning build artifacts..."
	@rm -f $(BINARY)
	@rm -rf $(BUILD_DIR)

install: build
	@echo "Installing to /usr/local/bin..."
	@sudo cp $(BINARY) /usr/local/bin/

release:
	@echo "Cross-compiling for release..."
	@mkdir -p $(BUILD_DIR)
	GOOS=linux GOARCH=amd64 go build -o $(BUILD_DIR)/$(BINARY)-linux-amd64 .
	GOOS=darwin GOARCH=arm64 go build -o $(BUILD_DIR)/$(BINARY)-darwin-arm64 .
	@echo "Release binaries created in $(BUILD_DIR)/"
