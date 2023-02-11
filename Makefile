# Execute `make clobber` after any target change
# TARGET = x86_64-unknown-linux-musl
 TARGET = x86_64-unknown-linux-gnu
#TARGET = aarch64-unknown-linux-gnu

DOCKER_IMAGES_INGRESS = ph_kafka_ingress transport_udp_send ph_mock_ingress ph_modbus_ingress ph_udp_ingress filter
DOCKER_IMAGES_EGRESS = ph_kafka_egress transport_udp_receive ph_mock_egress ph_modbus_egress ph_udp_egress filter

TARS_INGRESS = $(addprefix dockers/,$(addsuffix .tar,$(DOCKER_IMAGES_INGRESS)))
TARS_EGRESS = $(addprefix dockers/,$(addsuffix .tar,$(DOCKER_IMAGES_EGRESS)))

.PHONY: release binaries docker_images clean
.NOTPARALLEL: release binaries docker_images

release: Makefile binaries docker_images
	$(MAKE) scripts/osdd_ingress.tar.gz
	$(MAKE) scripts/osdd_egress.tar.gz

scripts/osdd_ingress.tar.gz: Makefile scripts/osdd.service target/release/osdd settings/ingress/Config.toml settings/egress/Config.toml $(addprefix scripts/,$(TARS_INGRESS))
	(cd scripts && tar -czvf osdd_ingress.tar.gz osdd.service ../target/$(TARGET)/release/osdd ../settings/ingress/Config.toml $(TARS_INGRESS) && cd ..)

scripts/osdd_egress.tar.gz: Makefile scripts/osdd.service target/release/osdd settings/egress/Config.toml settings/egress/Config.toml $(addprefix scripts/,$(TARS_EGRESS))
	(cd scripts && tar -czvf osdd_egress.tar.gz osdd.service ../target/$(TARGET)/release/osdd ../settings/egress/Config.toml $(TARS_EGRESS) && cd ..)

target/$(TARGET)/release/%: Makefile binaries

scripts/dockers/%.tar: Makefile target/$(TARGET)/release/%
	docker image rm $(notdir $(basename $@)) | true
	cp target/$(TARGET)/release/$(notdir $(basename $@)) scripts
ifeq ($(strip $(TARGET)),aarch64-unknown-linux-gnu)
	docker build --platform linux/arm64 -t $(notdir $(basename $@)) scripts --build-arg file=$(notdir $(basename $@));
else
	docker build -t $(notdir $(basename $@)) scripts --build-arg file=$(notdir $(basename $@));
endif
	rm $(notdir $(basename $@)) | true
	docker save $(notdir $(basename $@)) > scripts/dockers/$(notdir $(basename $@)).tar

binaries: Makefile
	rustup target add $(TARGET)
	cargo build --all --release
ifeq ($(strip $(TARGET)),aarch64-unknown-linux-gnu)
	RUSTFLAGS='-C target-feature=+crt-static -C linker=aarch64-linux-gnu-gcc' cargo build --target $(TARGET) --release
else
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --target $(TARGET) --release
endif

docker_images: Makefile
	$(MAKE) $(addsuffix .tar,$(addprefix scripts/dockers/,$(DOCKER_IMAGES_INGRESS)))
	$(MAKE) $(addsuffix .tar,$(addprefix scripts/dockers/,$(DOCKER_IMAGES_EGRESS)))
	rm $(addprefix scripts/,$(DOCKER_IMAGES_INGRESS)) $(addprefix scripts/,$(DOCKER_IMAGES_EGRESS)) 2> /dev/null | true

clean:
	rm -rf scripts/dockers/* scripts/osdd_ingress.tar.gz scripts/osdd_egress.tar.gz

distclean: clean
	rm -rf target/

clobber: distclean
	docker system prune -af
