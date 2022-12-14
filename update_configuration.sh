#!/usr/bin/env bash
if [ "$#" -ne 2 ]; then
	echo Arguments missing, exiting
	exit 1
fi
current_user="$1"
app="$2"
cat /home/${current_user}/.xscreensaver | tr '\n' '@' |  sed 's/\(.*\)\\n\\/\1\\n\\\n-\t\t\t\t'"${app}"' -root \t\t\t\\n\\/' | tr '@' '\n' > /home/${current_user}/${app}.tmp
mv /home/${current_user}/${app}.tmp /home/${current_user}/.xscreensaver
