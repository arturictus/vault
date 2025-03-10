###
# When trying to use yubikey library we require this binaries in the host machine.
# This approach seems to not work on macs. 
###
# environment setup
# required dependencies when installing this application

# brew install libsodium
# brew install pkg-config
# brew install pinentry-mac

# pkg-config --cflags --libs libsodium


# cd src-tauri && LIBRARY_PATH=/opt/homebrew/Cellar/libsodium/1.0.20/lib C_INCLUDE_PATH=/opt/homebrew/Cellar/libsodium/1.0.20/include cargo build