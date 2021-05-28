README.md: src/lib.rs
	echo "# willow" > README.md
	grep -P '^//!' src/lib.rs | cut -c5- >> README.md
