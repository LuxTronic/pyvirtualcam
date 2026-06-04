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
PYVER=${PYTHON_VERSION//.}

# Install package and test
${PYBIN}/pip install ./dist/pyvirtualcam*cp${PYVER}*manylinux*.whl

${PYBIN}/pip install -r dev-requirements.txt

# Device-dependent tests (test_camera.py, test_capture.py) require v4l2loopback
# and are not run in CI. Mock/utility tests validate the Python API contract.
${PYBIN}/pytest -v /io/test/test_backend_contract.py /io/test/test_util.py
