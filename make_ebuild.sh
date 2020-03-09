#!/bin/bash

echo "generating ebuild"
EBUILD="$( cargo ebuild | grep -oP "(?<=Wrote: ).*" )"
echo "'$EBUILD' generated"

# remove package from crate
echo "removing package from crates"
sed -i "/^project-chooser-[0-9.]*$/d" "$EBUILD"

# add repo to src_uri
echo "adding repo to SRC_URI"
REPO='https://gitlab.com/clotodex/${PN}/-/archive/${PV}/${PN}-${PV}.tar.bz2'
ESCAPED_REPO="$( echo $REPO | sed -e 's/[\/&]/\\&/g' )"
sed -i "s/SRC_URI=\"\(.*\)/SRC_URI=\"$ESCAPED_REPO \1/g" "$EBUILD"

# copy alias files
echo "adding bash alias to bin"
cat >>"$EBUILD" <<EOL
src_install() {
    cargo_src_install

    dobin bin/project-chooser-bash
}
EOL
echo "done"
