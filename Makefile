command := cargo
flags := build --release
build := $(command) $(flags)
app := e4biblesaver
target := target/release
install_dir := /usr/libexec/xscreensaver
xconfig_dir := /usr/share/xscreensaver/config
setting := "-	\\t\\t\\t $(app) -root \\t\\t\\t\\\n\\"
config_dir := /opt/$(app)

root := root
current_user := $(USER)
ifeq ($(current_user), $(root))
	current_user := $(SUDO_USER)
endif


all:
	$(build)

install:
	@sudo mkdir -p $(config_dir)
	sudo cp bible.txt $(config_dir)
	sudo cp $(target)/$(app) $(install_dir)
	sudo cp $(app).xml $(xconfig_dir)
	@if [ `grep -il "$(app)" /home/$(current_user)/.xscreensaver` ]; then\
		echo "The setting is already present in your configuration file" ;\
	else\
		./update_configuration.sh $(current_user) $(app) ;\
	fi

clean:
	$(command) clean

uninstall:
	sudo rm -rf $(config_dir)
	sudo rm -rf $(install_dir)/$(app)
	sudo rm $(xconfig_dir)/$(app).xml
	@echo Uninstalled!
