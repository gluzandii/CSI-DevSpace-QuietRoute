//! Safe pathfinding using A* algorithm with safety-weighted costs.
//!
//! This module implements the core routing logic that finds optimal paths
//! balancing both distance and safety. Uses A* algorithm with a safety penalty
//! multiplier to prefer well-lit, police-proximate routes.

use crate::models::RoadGraph;
use petgraph::algo::astar;
use petgraph::graph::NodeIndex;

/// Finds the safest path between two nodes in the road network.
///
/// Uses A* algorithm with a custom cost function that heavily prioritizes safety over distance.
/// The cost for each street segment is: `distance × (1 / safety_score)²`
///
/// This means:
/// - Safe streets (score 1.0): cost = distance (no penalty)
/// - Moderately safe (score 0.5): cost = distance × 4 (moderate penalty)
/// - Dangerous streets (score 0.1): cost = distance × 100 (extremely penalized)
///
/// The squared penalty means unsafe routes are exponentially more expensive,
/// so the algorithm strongly prefers routes with better lighting and police proximity.
///
/// # Arguments
/// * `graph` - The road network graph with safety-scored edges
/// * `start_idx` - Starting intersection node index
/// * `end_idx` - Destination intersection node index
///
/// # Returns
/// * `Some((cost, path))` - The safety-weighted cost and sequence of nodes from start to end
/// * `None` - If no path exists (disconnected graph components)
///
/// # Cost Interpretation
/// If cost ≈ distance: Route is very safe (excellent police/lighting coverage)
/// If cost >> distance: Route includes unsafe segments (poor safety infrastructure)
///
/// # Performance
/// A* with straight-line heuristic explores far fewer nodes than Dijkstra,
/// typically 20-100x faster for geographic routing.
pub fn find_safe_path(
    graph: &RoadGraph,
    start_idx: NodeIndex,
    end_idx: NodeIndex,
) -> Option<(f64, Vec<NodeIndex>)> {
    astar(
        graph,
        start_idx,
        |finish| finish == end_idx,
        |edge_ref| {
            let edge = edge_ref.weight();

            // SAFETY-PRIORITIZED COST CALCULATION
            // We square the safety penalty to heavily penalize unsafe streets.
            // This makes the algorithm prefer safer routes even if they're longer.
            // 1.0 (Safe) -> multiplier of 1.0
            // 0.5 (Medium) -> multiplier of 4.0
            // 0.1 (Dangerous) -> multiplier of 100.0
            let safety_penalty = 1.0 / edge.safety_score.max(0.1);
            let safety_penalty_squared = safety_penalty * safety_penalty;

            edge.distance_meters * safety_penalty_squared
        },
        |node_idx| {
            // Heuristic: Straight-line distance to goal
            // This helps A* run much faster by "guessing" the direction.
            let start = graph[node_idx].coord;
            let end = graph[end_idx].coord;
            utils::geo::haversine_distance(start.lat, start.lon, end.lat, end.lon)
        },
    )
}
