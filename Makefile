command := cargo
flags := build --release
build := $(command) $(flags)
app := e4biblesaver
bible := bible
target := target/release
home_dir := $(shell pwd)

all:
	$(build)
	cp -R $(bible) $(target)
	cd $(target) ; zip -9 -r $(home_dir)/$(app).zip ./$(app) ./$(bible)
	cd $(home_dir)
	@echo $(app).zip correctly built

clean:
	$(command) clean
