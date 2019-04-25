#!/bin/bash
cargo build --release || exit 1
su -c "cp target/release/project-chooser /usr/bin/; chmod 755 /usr/bin/project-chooser"

#TODO copy aliases to /etc/bash/bashrc.d/99-project-chooser.sh
