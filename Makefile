command := cargo
flags := build --release
build := $(command) $(flags)
app := e4biblesaver
install_dir := /opt/$(app)
bible := $(install_dir)/bible.txt
bin := /usr/bin

all: bible_h
	$(build)

timestamp: timestamp.txt

bible_h:
	echo '"$(bible)"' > src/bible.h

install:
	sudo rm -rf $(install_dir)
	sudo rm -f $(bin)/$(app)
	sudo install -d $(install_dir)
	sudo install target/release/e4biblesaver $(install_dir)/$(app)
	sudo install bible.txt $(bible)
	sudo ln -s $(install_dir)/$(app) $(bin)/$(app)