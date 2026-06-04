#!/bin/bash
set -e -x

# List python versions
ls /opt/python

if [ $PYTHON_VERSION == "3.8" ]; then
    PYBIN="/opt/python/cp38-cp38/bin"
elif [ $PYTHON_VERSION == "3.9" ]; then
    PYBIN="/opt/python/cp39-cp39/bin"
elif [ $PYTHON_VERSION == "3.10" ]; then
    PYBIN="/opt/python/cp310-cp310/bin"
elif [ $PYTHON_VERSION == "3.11" ]; then
    PYBIN="/opt/python/cp311-cp311/bin"
elif [ $PYTHON_VERSION == "3.12" ]; then
    PYBIN="/opt/python/cp312-cp312/bin"
elif [ $PYTHON_VERSION == "3.13" ]; then
    PYBIN="/opt/python/cp313-cp313/bin"
else
    echo "Unsupported Python version $PYTHON_VERSION"
    exit 1
fi

if [ ! -z "$GITHUB_ENV" ]; then 
    echo "CODEQL_PYTHON=$PYBIN/python" >> $GITHUB_ENV
    echo "PATH=$PYBIN:$PATH" >> $GITHUB_ENV
fi

# Install Rust for the Linux backend.
if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
fi
source "$HOME/.cargo/env"
rustc --version
cargo --version

# install compile-time dependencies
${PYBIN}/pip install numpy==${NUMPY_VERSION}
${PYBIN}/pip install setuptools setuptools-rust

# List installed packages
${PYBIN}/pip freeze

# Build pyvirtualcam wheel
export LDFLAGS="-Wl,--strip-debug"
${PYBIN}/python setup.py bdist_wheel --dist-dir dist-tmp

# Bundle external shared libraries into wheel and fix the wheel tags
mkdir dist
auditwheel repair dist-tmp/pyvirtualcam*.whl -w dist
ls -al dist
