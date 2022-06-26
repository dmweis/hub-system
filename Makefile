TARGET_URL ?= homepi.local
TARGET_HOST ?= pi@$(TARGET_URL)
REMOTE_DIRECTORY ?= /home/pi
ARM_BUILD_PATH ?= target/debian/hub-system*.deb


.PHONY: build
build:
	cargo build --release
	cargo deb --no-build

.PHONE: install
install: build
	sudo dpkg -i $(ARM_BUILD_PATH)

.PHONY: deploy
deploy: build
	@echo "Sending $(ARM_BUILD_PATH) to $(TARGET_HOST):$(REMOTE_DIRECTORY)"
	rsync -avz --delete $(ARM_BUILD_PATH) $(TARGET_HOST):$(REMOTE_DIRECTORY)

.PHONY: debug
debug:
	cargo run
