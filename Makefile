.PHONY: readme

readme: README.md

README.md: src/lib.rs
	@ cargo readme > README.md
	@ sed -i.back 's/\[\(`[^`]*`\)]/\1/g' README.md
	@ rm readme.md.back
