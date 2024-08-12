#!/bin/bash

mkdir -p assets

# https://gitlab.com/mattbas/python-lottie/
# pipx install lottie
# pipx inject lottie setuptools
lottie_convert.py assets_src/bird_ready.sif assets/bird_ready.json

# this one: https://github.com/ed-asriyan/lottie-converter
$LOTTIE_CONVERTER_PATH/lottie-converter/lottie_to_gif.sh assets/bird_ready.json