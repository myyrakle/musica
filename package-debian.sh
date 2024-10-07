cargo build --release 
mv target/release/musica-app dist/debian-package/usr/local/bin/musica
dpkg-deb --build dist/debian-package dist/musica.deb