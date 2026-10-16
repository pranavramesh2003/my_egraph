# my_egraph
Implementation of a simple egraph in Rust following [Pavel Panchenka's blog](https://pavpanchekha.com/blog/microegg.html). This mirrors the Python implementation considerably, since this was my first Rust project

## The union-find data structure

We store the parent map as a vector of IDs (integers), and traversing this vector would yield the parent of each node. `union` ensures that both the IDs have the same parent. Path compression is *not* performed, since the parent choice is arbitrary. 

## The E-graph

The e-graph stores an underlying union-find data structure, as well as a HashCons map stored as a `HashMap`. It has the ability to perform top-down E-matching, instantiate patterns, and perform rewrites. Upon rebuilding, the E-graph is brought into a consistent (correct) state. Rebuilding can be performed as and when necessary


## The tests

The tests check if the basic E-graph operations like `add` work satisfactorily, and also tests the E-matcher and rewriter with an example. 