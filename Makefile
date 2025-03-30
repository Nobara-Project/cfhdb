all:
	true

build:
	cargo fetch
	cargo build --release

build_debug:
	cargo fetch
	cargo build

install_no_build:
	mkdir -p $(DESTDIR)/usr/bin/
	cp -vf target/release/cfhdb $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/cfhdb
	mkdir -p $(DESTDIR)/usr/lib/cfhdb/
	cp -rvf data/scripts $(DESTDIR)/usr/lib/cfhdb/
	mkdir -p $(DESTDIR)/etc/cfhdb/
	cp -rvf data/profile-config.json $(DESTDIR)/etc/cfhdb/
	chmod 755 $(DESTDIR)/usr/lib/cfhdb/scripts/*.sh
	cp -rvf data/polkit-1 $(DESTDIR)/usr/share/

install_no_build_debug:
	mkdir -p $(DESTDIR)/usr/bin/
	cp -vf target/debug/cfhdb $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/cfhdb
	mkdir -p $(DESTDIR)/usr/lib/cfhdb/
	cp -rvf data/scripts $(DESTDIR)/usr/lib/cfhdb/
	mkdir -p $(DESTDIR)/etc/cfhdb/
	cp -rvf data/profile-config.json $(DESTDIR)/etc/cfhdb/
	chmod 755 $(DESTDIR)/usr/lib/cfhdb/scripts/*.sh
	cp -rvf data/polkit-1 $(DESTDIR)/usr/share/

install:
	mkdir -p $(DESTDIR)/usr/bin/
	cargo fetch
	cargo build --release
	cp -vf target/release/cfhdb $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/cfhdb
	mkdir -p $(DESTDIR)/usr/lib/cfhdb/
	cp -rvf data/scripts $(DESTDIR)/usr/lib/cfhdb/
	mkdir -p $(DESTDIR)/etc/cfhdb/
	cp -rvf data/profile-config.json $(DESTDIR)/etc/cfhdb/
	chmod 755 $(DESTDIR)/usr/lib/cfhdb/scripts/*.sh
	cp -rvf data/polkit-1 $(DESTDIR)/usr/share/