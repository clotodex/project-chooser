#!/bin/bash
cargo build --release || exit 1
su -c "cp target/release/project-chooser /usr/bin/;\
	   chmod 755 /usr/bin/project-chooser;\
	   cp target/release/recache /usr/bin/pc-recache;
	   chmod 755 /usr/bin/pc-recache;"

#TODO copy aliases to /etc/bash/bashrc.d/99-project-chooser.sh
