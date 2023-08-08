test-txt:
	echo "Hello, world!" > test.txt

test-bin:
	dd if=/dev/urandom of=test.bin bs=32k count=4 ; sync

test-dir:
	mkdir -p test/{1..10}
	zsh -c "echo hi > test/{1..10}/{1..3}.txt"

test: test-txt test-dir
	cargo test
