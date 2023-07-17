test-txt:
	echo "Hello, world!" > test.txt

test-dir:
	mkdir -p test/

test: test-txt test-dir
	cargo test