#!/bin/bash
# has to be executed from the root directory. To lazy for adding a check for this right now

git clone https://github.com/mathjax/MathJax.git mj-tmp
mkdir -p theme/js/mathjax
mv mj-tmp/es5 theme/js/mathjax
rm -rf mj-tmp

