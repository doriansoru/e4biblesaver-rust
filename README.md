A xscreensaver module, written in rust, to select a biblical random verse from a text file which contains the Italian version of the bible in the format:

    book|chapter number|verse number|verse or full sentence text

The command `make` builds the program. `make install` installs it in `/opt/e4biblesaver`, in `/usr/libexec/xscreensaver`, in `/usr/libexec/xscreensaver/config` and tries to update `~/.xscreensaver` to add this module. 

`rust` compiler, `xscreensaver`, `libx11-dev` and `libxft-dev` are required.  
