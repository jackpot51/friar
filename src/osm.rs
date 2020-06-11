use osmpbfreader::{Error, OsmPbfReader, OsmObj, Node, NodeId, Result, Way, WayId};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub struct Osm {
    pub nodes: HashMap<NodeId, Node>,
    pub ways: HashMap<WayId, Way>,
}

impl Osm {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Osm> {
        let mut nodes = HashMap::new();
        let mut ways = HashMap::new();

        let file = File::open(p).map_err(|err| Error::Io(err))?;
        for obj_res in OsmPbfReader::new(file).iter() {
            match obj_res? {
                OsmObj::Node(node) => {
                    nodes.insert(node.id, node);
                },
                OsmObj::Way(way) => {
                    ways.insert(way.id, way);
                },
                _ => ()
            }
        }

        Ok(Osm {
            nodes,
            ways,
        })
    }
}
