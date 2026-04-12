#!/bin/bash
# Copie la bibliothèque compilée vers excel/Interop/
mkdir -p ../excel/Interop/

if [ "$(uname)" == "Linux" ]; then
    cp target/release/libftp_core_bindings_c.so ../excel/Interop/
elif [ "$(uname)" == "Darwin" ]; then
    cp target/release/libftp_core_bindings_c.dylib ../excel/Interop/
else  # Windows
    cp target/release/ftp_core_bindings_c.dll ../excel/Interop/
fi
