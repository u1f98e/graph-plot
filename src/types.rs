use bevy::prelude::*;

use crate::graph::{GEdge, GNode};

pub type GNodeExclusive = (With<GNode>, Without<GEdge>);
pub type GEdgeExclusive = (With<GEdge>, Without<GNode>);