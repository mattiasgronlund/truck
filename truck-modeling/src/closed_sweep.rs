use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Vertex<P> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        _: &FC,
        _: &FS,
        connect_points: &CP,
        _: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut wire = Wire::new();
        let mut vertex = self.clone();
        for _ in 1..division {
            let new_vertex = vertex.mapped(point_mapping);
            wire.push_back(connect_vertices(&vertex, &new_vertex, connect_points));
            vertex = new_vertex;
        }
        wire.push_back(connect_vertices(&vertex, self, connect_points));
        wire
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Edge<P, C> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        _: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut shell = Shell::new();
        let mut edge = self.clone();
        for _ in 1..division {
            let new_edge = edge.mapped(point_mapping, curve_mapping);
            shell.push(connect_edges(
                &edge,
                &new_edge,
                connect_points,
                connect_curves,
            ));
            edge = new_edge;
        }
        shell.push(connect_edges(&edge, self, connect_points, connect_curves));
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Wire<P, C> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        _: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut shell = Shell::new();
        let mut wire = self.clone();
        for _ in 1..division {
            let new_wire = wire.mapped(point_mapping, curve_mapping);
            shell.extend(connect_wires(
                &wire,
                &new_wire,
                connect_points,
                connect_curves,
            ));
            wire = new_wire;
        }
        shell.extend(connect_wires(&wire, self, connect_points, connect_curves));
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Face<P, C, S> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        let boundaries: Vec<_> = self
            .boundaries()
            .iter()
            .map(move |wire| {
                let mut shell = wire.closed_sweep(
                    point_mapping,
                    curve_mapping,
                    surface_mapping,
                    connect_points,
                    connect_curves,
                    division,
                );
                if !self.orientation() {
                    shell.iter_mut().for_each(|face| {
                        face.invert();
                    });
                }
                shell
            })
            .collect();
        Solid::debug_new(boundaries)
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Shell<P, C, S> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        self.connected_components()
            .into_iter()
            .map(move |shell| {
                let bdry = shell
                    .extract_boundaries()
                    .iter()
                    .map(|wire| {
                        wire.closed_sweep(
                            point_mapping,
                            curve_mapping,
                            surface_mapping,
                            connect_points,
                            connect_curves,
                            division,
                        )
                    })
                    .collect();
                Solid::try_new(bdry)
            })
            .collect()
    }
}
