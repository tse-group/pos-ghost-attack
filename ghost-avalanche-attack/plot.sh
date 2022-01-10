#! /bin/bash -ve

# for "dot" see: https://graphviz.org/ https://en.wikipedia.org/wiki/DOT_(graph_description_language)

cargo build --bin attack-pos-ghost --features shortscenario
./target/debug/attack-pos-ghost 2> attack-pos-ghost.dot
dot -Tpng attack-pos-ghost.dot > attack-pos-ghost.png

cargo build --bin attack-committee-ghost --features shortscenario
./target/debug/attack-committee-ghost 2> attack-committee-ghost.dot
dot -Tpng attack-committee-ghost.dot > attack-committee-ghost.png

