Color Smash
===========

Color Smash reduces the number of colors in an image to 256, so it could be stored as indexes into a palette.  It can also convert a group of images such that they could be stored as a single set of indexes into a different palette for each image.

This allows efficient storage of images that are basically palette swaps, even for complicated images.  For example, if you have versions of a model that differ only by changing color of the textures, renders of those models could be stored as palette swaps.

Algorithm
---------

Currently Color Smash uses the k-means algorithm, with the distance between two colors calculated as described at http://www.imagemagick.org/Usage/bugs/fuzz_distance/.

The initial points are chosen by finding the cluster with the greatest total distance to all nodes, and then placing a new centroid at the node furthest from it, and doing so repeatedly.  In my testing this worked better than random initialization or k-means++.  (Note that I'm optimizing for output quality, not speed.)
