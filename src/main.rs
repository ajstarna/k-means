use rand::Rng;
use clap::{App, Arg};
use std::sync::mpsc;
use crossbeam;


// A struct with an x, y coord
#[derive(Debug, PartialEq, PartialOrd)]
struct Point{
    x: f64,
    y: f64,
}


impl Point {
    /// Construct a ranomd point within the given bounds
    fn new_random_within_range(left: f64, right: f64, bottom: f64, top: f64) -> Self {
	let mut rng = rand::thread_rng();
	let x = rng.gen_range(left..=right);
	let y = rng.gen_range(bottom..=top);
	Point { x, y }
    }

    /// sqaured Euclidean distance between two points
    /// we swuare to save computation, and doesn't affect our final answer of comparing which points are closer together
    fn squared_distance(p1: &Point, p2: &Point) -> f64 {
	(p1.x - p2.x).powf(2.0) + (p1.y - p2.y).powf(2.0)
    }
    
    /// Given a list of clusters, we place ourself into the cluster with the closest centroid
    fn find_best_cluster(&self, clusters: & Vec<Cluster>) -> usize{
	let mut best_idx = 0;
	let mut best_distance = f64::INFINITY;
	for (i, cluster) in clusters.iter().enumerate() {
	    let current_compare = Point::squared_distance(&self, &cluster.centroid);
	    if current_compare < best_distance {
		best_distance = current_compare;
		best_idx = i;
	    }
	}
	best_idx
    }
}

#[derive(Debug)]
struct Cluster<'a> {
    centroid: Point, // the centroid, i.e. the arithmetic mean of all consituent points
    points: Vec<&'a Point>,
}

impl <'a> Cluster <'a>
{
    /// the new_random function creates an empty cluster with a random centroid
    fn new_random (left: f64, right: f64, bottom: f64, top: f64) -> Self {
	Cluster {
	    centroid: Point::new_random_within_range(left, right, bottom, top),
	    points: vec![],
	}
    }

    fn clear_points(&mut self) {
    	self.points.clear();
    }

    /// This method iterates over all points within the cluster and updates self.centroid to be
    /// the arithmetic mean of all points.
    /// It returns how much the centroid was changed from the initial value before the function was called.
    fn set_centroid(&mut self) -> f64{
	if self.points.len() == 0 {
	    return 0.;
	}
	let mut sum_x = 0.;
	let mut sum_y = 0.;	
	for point in & self.points {
	    sum_x += point.x;
	    sum_y += point.y;	    
	}
	let new_centroid = Point {x: sum_x / (self.points.len() as f64), y: sum_y / (self.points.len() as f64) };
	let change = Point::squared_distance(&self.centroid, &new_centroid);
	(*self).centroid = new_centroid;
	change
    }
}


fn cluster_points<'a>(points: &'a Vec<Point>, num_clusters: usize, left: f64, right: f64, bottom: f64, top: f64, num_threads: usize)
		      -> Vec<Cluster<'a>> {

    // Initialize clusters with random centroids
    let mut clusters = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
	let cluster = Cluster::new_random(left, right, bottom, top);
	clusters.push(cluster);
    }

    println!("Clusters to begin: {:?}", clusters);
    
    const EPSILON: f64 = 0.05; // this defines the threshold for when the clusters have converged
    

    // We construct as many chunks of the points vector as there are threads.
    // For each interation in the loop, one thread will be responsible for all
    // the points in a given chunk.
    let point_chunks: Vec<& [Point] > = points.chunks(num_threads).collect();
    let (sender, receiver) = mpsc::channel(); // when a point has found its best cluster, pass that info in the channel


    let mut change = f64::INFINITY; // the overall change of all clusters' centroids    
    // While the cluster centroids are still changing "enough", we keep re-assigning the points
    while change > EPSILON {
	for cluster in &mut clusters {
	    // at the start of each loop, we clear all points from each cluster
	    // so that they can be re-assigned to their closest cluster
	   cluster.clear_points();
	}
	{
	    let clusters_ref = &clusters; // clusters_ref lets us move a reference to clusters into each thread
	    for chunk in &point_chunks {
		let sender_n = sender.clone(); // each thread needs its own clone of the sender
		crossbeam::scope(|spawner| {
		    // crosbeam scope ensures that all threads will be done before we move on,
		    // this lets us safely borrow the points and clusters without them needing
		    // a 'static lifetime
		    spawner.spawn(move |_| {
			for point in *chunk {	
			    // find the best cluster for each point, and send the info into the channel
			    let  best_idx = point.find_best_cluster(clusters_ref);
			    sender_n.send((point, best_idx)).unwrap();
			}
		    });
		}).unwrap();
	    }
	}

	// we iterate through each point assignment as grabbed from the receiver end of the channel
	// and place each point into the corresponding cluster
	let mut results_received = 0;
	for (point, best_idx) in &receiver {
	    results_received += 1;
	    clusters[best_idx].points.push(point);
	    if results_received >= points.len() {
		// if we got a result for every point, then we are done
		break;
	    }
	}

	change = 0.0;	
	for cluster in &mut clusters {
	    // Now that the points have been assigned, tell the clusters
	    // to recalculate their centroids, and return how much of a change there was
	    change += cluster.set_centroid();
	}
	println!("change = {}", change);
    }
    clusters
}    


fn main() {
    let matches = App::new("myapp")
        .version("1.0")
        .author("Adam S. <ajstana@ualberta.ca>")
        .about("K centroids cluster")
        .arg(Arg::with_name("num_points")
            .short("p")
            .long("num_points")
            //.value_name("NUM_POINTS")
             .help("The number of random points to cluster")
	     .required(true)	     
             .takes_value(true))
        .arg(Arg::with_name("num_clusters")
             .short("c")
             .long("num_clusters")
             //.value_name("NUM_CLUSTERS")
             .help("The number of clusters to use")
	     .required(true)
            .takes_value(true))
        .arg(Arg::with_name("num_threads")
             .short("t")
             .long("num_threads")
             //.value_name("NUM_CLUSTERS")
             .help("The number of threads to use")
	     .required(false)
	     .default_value("4")
            .takes_value(true))
        .get_matches();


    // since both args are requied, we are free to unwrap
    let num_clusters: usize = matches.value_of("num_clusters").unwrap().parse().unwrap();
    let num_points: usize = matches.value_of("num_points").unwrap().parse().unwrap();
    let num_threads: usize = matches.value_of("num_threads").unwrap().parse().unwrap();     



    // These constants define the boundary in the real plane where the points and clusters can exist
    const LEFT: f64 = -5.;
    const RIGHT: f64 = 5.;
    const BOTTOM: f64 = -5.;
    const TOP: f64 = 5.;


    // initialize our random points that will be clustered
    let mut points = Vec::with_capacity(num_points);
    for _ in 0..num_points {
	let point = Point::new_random_within_range(LEFT, RIGHT, BOTTOM, TOP);
	points.push(point);
    }

    // call to function to cluster the points
    let clusters = cluster_points(&points, num_clusters, LEFT, RIGHT, BOTTOM, TOP, num_threads);

    for (i, cluster) in clusters.iter().enumerate() {
	println!("Cluster {} has centroid at {:?} and {} points", i, cluster.centroid, cluster.points.len());
    }
	

}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_set_mean() {
	let centroid = Point{ x: 3., y: 4. };
	let points = vec![&Point{ x: 1., y: 1. }, &Point{ x: -1., y: -1. }] ;
	let mut cluster = Cluster { centroid, points: points };
	let diff = cluster.set_centroid();
	assert_eq!(cluster.centroid, Point{ x: 0., y: 0. });
	// the new centroid will be (0, 0) which is 55 away in squared euclidean distance from (3, 4)
	assert_eq!(diff, 25.0 );
    }
}
