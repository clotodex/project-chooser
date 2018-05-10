# This file contains useful aliases for using the projectchooser
# source this file in you bashrc to use these aliases
# WARNING: projectchooser has to be in the PATH

function pccd(){
	dir="$(project-chooser $*)" && cd "$dir"
}
export pccd

alias "pc"='pccd'
alias "pcb"='pccd -b'
