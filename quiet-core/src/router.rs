use crate::models::RoadGraph;
use petgraph::algo::astar;
use petgraph::graph::NodeIndex;

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

            // --- THE MULTIPLIER ---
            // We use the safety_score (0.1 to 1.0) we calculated earlier.
            // 1.0 (Safe) -> multiplier of 1.0
            // 0.1 (Dangerous) -> multiplier of 10.0
            let safety_penalty = 1.0 / edge.safety_score.max(0.1);

            edge.distance_meters * safety_penalty
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
