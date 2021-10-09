# k-means
This is my very first rust project, so I thought I would attempt k means clustering https://en.wikipedia.org/wiki/K-means_clustering

Given c clusters and p points, the code intializes p random points and c random clusters and iteratively places the points into their best match cluster until convergence.
Each cluster has a centroid (the arithmetic mean of all consituent points), and we use normal Euclidean distance for the distance between two points.

This project uses the rust library clap for handling command line arguments and the the library crossbeam for helping with threads.

usage:
cargo run --  -c 3 -p 100000 -t 6 # this will cluster 100,000 points into 3 threads and use 6 threads.

TODO: the calculation of the points' best clusters is done in parallel, but the point assignment and calculation of cluster centroids happens synchronously. 
I could parallelize the set_centroid() method, and/or better yet have the assignment and centroid update happen as each point assignment is ready. 
This would require the receiver end of the channel to not wait for the threads to be done. I am not sure how to do that with crossbeam atm.
