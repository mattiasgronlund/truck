use crate::errors::Error;
use crate::shell::ShellCondition;
use crate::*;
use std::vec::Vec;

impl<P, C, S> Solid<P, C, S> {
    /// create the shell whose boundaries is boundary.
    /// # Panic
    /// All boundary must be non-empty, connected, and closed manifold.
    #[inline(always)]
    pub fn new(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> {
        Solid::try_new(boundaries).remove_try()
    }
    /// create the shell whose boundaries is boundary.
    /// # Failure
    /// All boundary must be non-empty, connected, and closed manifold.
    #[inline(always)]
    pub fn try_new(boundaries: Vec<Shell<P, C, S>>) -> Result<Solid<P, C, S>> {
        for shell in &boundaries {
            if shell.is_empty() {
                return Err(Error::EmptyShell);
            } else if !shell.is_connected() {
                return Err(Error::NotConnected);
            } else if shell.shell_condition() != ShellCondition::Closed {
                return Err(Error::NotClosedShell);
            } else if !shell.singular_vertices().is_empty() {
                return Err(Error::NotManifold);
            }
        }
        Ok(Solid::new_unchecked(boundaries))
    }
    /// create the shell whose boundaries is boundary.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether all boundary is non-empty, connected, and closed.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> {
        Solid {
            boundaries: boundaries,
        }
    }

    /// create the shell whose boundaries is boundary.
    /// # Remarks
    /// This method checks whether all boundary is non-empty, connected, and closed in the debug mode.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn debug_new(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> {
        match cfg!(debug_assertions) {
            true => Solid::new(boundaries),
            false => Solid::new_unchecked(boundaries),
        }
    }

    /// Returns the reference of boundary shells
    #[inline(always)]
    pub fn boundaries(&self) -> &Vec<Shell<P, C, S>> { &self.boundaries }
    /// Returns the boundary shells
    #[inline(always)]
    pub fn into_boundaries(self) -> Vec<Shell<P, C, S>> { self.boundaries }

    /// Returns an iterator over the faces.
    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a Face<P, C, S>> {
        self.boundaries.iter().flatten()
    }

    /// Returns an iterator over the edges.
    #[inline(always)]
    pub fn edge_iter<'a>(&'a self) -> impl Iterator<Item = Edge<P, C>> + 'a {
        self.face_iter().flat_map(Face::boundaries).flatten()
    }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter<'a>(&'a self) -> impl Iterator<Item = Vertex<P>> + 'a {
        self.edge_iter().map(|edge| edge.front().clone())
    }

    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn try_mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
        mut curve_mapping: impl FnMut(&C) -> Option<D>,
        mut surface_mapping: impl FnMut(&S) -> Option<T>,
    ) -> Option<Solid<Q, D, T>> {
        Some(Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |shell| {
                    shell.try_mapped(&mut point_mapping, &mut curve_mapping, &mut surface_mapping)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
    }

    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Q,
        mut curve_mapping: impl FnMut(&C) -> D,
        mut surface_mapping: impl FnMut(&S) -> T,
    ) -> Solid<Q, D, T> {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |shell| {
                    shell.mapped(&mut point_mapping, &mut curve_mapping, &mut surface_mapping)
                })
                .collect(),
        )
    }

    /// Cuts one edge into two edges at vertex.
    #[inline(always)]
    pub fn cut_edge(&mut self, edge_id: EdgeID<C>, vertex: &Vertex<P>) -> bool
    where
        P: Clone,
        C: Cut<Point = P> + SearchParameter<Point = P, Parameter = f64>, {
        let res = self
            .boundaries
            .iter_mut()
            .all(|shell| shell.cut_edge(edge_id, vertex));
        #[cfg(debug_assertions)]
        Solid::new(self.boundaries.clone());
        res
    }
    /// Removes `vertex` from `self` by concat two edges on both sides.
    #[inline(always)]
    pub fn remove_vertex_by_concat_edges(&mut self, vertex_id: VertexID<P>) -> bool
    where
        P: std::fmt::Debug,
        C: Concat<C, Point = P, Output = C> + Invertible + ParameterTransform, {
        let res = self
            .boundaries
            .iter_mut()
            .all(|shell| shell.remove_vertex_by_concat_edges(vertex_id));
        #[cfg(debug_assertions)]
        Solid::new(self.boundaries.clone());
        res
    }
}

impl<P, C, S> Solid<P, C, S>
where
    P: Tolerance,
    C: ParametricCurve<Point = P>,
    S: IncludeCurve<C>,
{
    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool {
        self.boundaries()
            .iter()
            .all(|shell| shell.is_geometric_consistent())
    }
}

#[allow(dead_code)]
pub(super) fn cube() -> Solid<(), (), ()> {
    use crate::*;
    use std::iter::FromIterator;
    let v = Vertex::news(&[(); 8]);
    let edge = [
        Edge::new(&v[0], &v[1], ()), // 0
        Edge::new(&v[1], &v[2], ()), // 1
        Edge::new(&v[2], &v[3], ()), // 2
        Edge::new(&v[3], &v[0], ()), // 3
        Edge::new(&v[0], &v[4], ()), // 4
        Edge::new(&v[1], &v[5], ()), // 5
        Edge::new(&v[2], &v[6], ()), // 6
        Edge::new(&v[3], &v[7], ()), // 7
        Edge::new(&v[4], &v[5], ()), // 8
        Edge::new(&v[5], &v[6], ()), // 9
        Edge::new(&v[6], &v[7], ()), // 10
        Edge::new(&v[7], &v[4], ()), // 11
    ];

    let wire0 = Wire::from_iter(vec![&edge[0], &edge[1], &edge[2], &edge[3]]);
    let face0 = Face::new(vec![wire0], ());

    let wire1 = Wire::from_iter(vec![
        &edge[4],
        &edge[8],
        &edge[5].inverse(),
        &edge[0].inverse(),
    ]);
    let face1 = Face::new(vec![wire1], ());

    let wire2 = Wire::from_iter(vec![
        &edge[5],
        &edge[9],
        &edge[6].inverse(),
        &edge[1].inverse(),
    ]);
    let face2 = Face::new(vec![wire2], ());

    let wire3 = Wire::from_iter(vec![
        &edge[6],
        &edge[10],
        &edge[7].inverse(),
        &edge[2].inverse(),
    ]);
    let face3 = Face::new(vec![wire3], ());
    let wire4 = Wire::from_iter(vec![
        &edge[7],
        &edge[11],
        &edge[4].inverse(),
        &edge[3].inverse(),
    ]);
    let face4 = Face::new(vec![wire4], ());
    let wire5 = Wire::from_iter(vec![
        &edge[11].inverse(),
        &edge[10].inverse(),
        &edge[9].inverse(),
        &edge[8].inverse(),
    ]);
    let face5 = Face::new(vec![wire5], ());

    let mut shell = Shell::new();
    shell.push(face0);
    shell.push(face5);
    assert!(!shell.is_connected());
    shell.push(face1);
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    assert!(shell.is_connected());
    shell.push(face2);
    shell.push(face3);
    shell.push(face4);

    Solid::new(vec![shell])
}

#[test]
fn cube_test() { cube(); }
